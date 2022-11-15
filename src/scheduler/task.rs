// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use crate::arch;
use crate::arch::processor::msb;
use crate::arch::{BasePageSize, PageSize};
use crate::consts::*;
use crate::logging::*;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt;

/// The status of the task - used for scheduling
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
	TaskInvalid,
	TaskReady,
	TaskRunning,
	TaskBlocked,
	TaskFinished,
	TaskIdle,
}

/// Unique identifier for a task (i.e. `pid`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
	pub const fn into(self) -> u32 {
		self.0
	}

	pub const fn from(x: u32) -> Self {
		TaskId(x)
	}
}

impl alloc::fmt::Display for TaskId {
	fn fmt(&self, f: &mut fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Priority of a task
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskPriority(u8);

impl TaskPriority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		TaskPriority(x)
	}
}

impl alloc::fmt::Display for TaskPriority {
	fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

pub const REALTIME_PRIORITY: TaskPriority = TaskPriority::from(NO_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: TaskPriority = TaskPriority::from(24);
pub const NORMAL_PRIORITY: TaskPriority = TaskPriority::from(16);
pub const LOW_PRIORITY: TaskPriority = TaskPriority::from(0);

struct QueueHead {
	head: Option<Rc<RefCell<Task>>>,
	tail: Option<Rc<RefCell<Task>>>,
}

impl QueueHead {
	pub const fn new() -> Self {
		QueueHead {
			head: None,
			tail: None,
		}
	}
}

impl Default for QueueHead {
	fn default() -> Self {
		Self {
			head: None,
			tail: None,
		}
	}
}

/// Realize a priority queue for tasks
pub struct PriorityTaskQueue {
	queues: [QueueHead; NO_PRIORITIES],
	prio_bitmap: u64,
}

impl PriorityTaskQueue {
	/// Creates an empty priority queue for tasks
	pub fn new() -> PriorityTaskQueue {
		PriorityTaskQueue {
			queues: Default::default(),
			prio_bitmap: 0,
		}
	}

	/// Add a task by its priority to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		let i = task.borrow().prio.into() as usize;
		//assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

		self.prio_bitmap |= 1 << i;
		match self.queues[i].tail {
			None => {
				// first element in the queue
				self.queues[i].head = Some(task.clone());

				let mut borrow = task.borrow_mut();
				borrow.next = None;
				borrow.prev = None;
			}
			Some(ref mut tail) => {
				// add task at the end of the node
				tail.borrow_mut().next = Some(task.clone());

				let mut borrow = task.borrow_mut();
				borrow.next = None;
				borrow.prev = Some(tail.clone());
			}
		}

		self.queues[i].tail = Some(task.clone());
	}

	fn pop_from_queue(&mut self, queue_index: usize) -> Option<Rc<RefCell<Task>>> {
		let new_head;
		let task;

		match self.queues[queue_index].head {
			None => {
				return None;
			}
			Some(ref mut head) => {
				let mut borrow = head.borrow_mut();

				match borrow.next {
					Some(ref mut nhead) => {
						nhead.borrow_mut().prev = None;
					}
					None => {}
				}

				new_head = borrow.next.clone();
				borrow.next = None;
				borrow.prev = None;

				task = head.clone();
			}
		}

		self.queues[queue_index].head = new_head;
		if self.queues[queue_index].head.is_none() {
			self.queues[queue_index].tail = None;
			self.prio_bitmap &= !(1 << queue_index as u64);
		}

		Some(task)
	}

	/// Pop the task with the highest priority from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			return self.pop_from_queue(i as usize);
		}

		None
	}

	/// Pop the next task, which has a higher or the same priority as `prio`
	pub fn pop_with_prio(&mut self, prio: TaskPriority) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = msb(self.prio_bitmap) {
			if i >= prio.into() as u64 {
				return self.pop_from_queue(i as usize);
			}
		}

		None
	}

	/// Remove a specific task from the priority queue.
	pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
		let i = task.borrow().prio.into() as usize;
		//assert!(i < NO_PRIORITIES, "Priority {} is too high", i);

		let mut curr = self.queues[i].head.clone();
		let mut next_curr;

		loop {
			match curr {
				None => {
					break;
				}
				Some(ref curr_task) => {
					if Rc::ptr_eq(&curr_task, &task) {
						let (mut prev, mut next) = {
							let borrowed = curr_task.borrow_mut();
							(borrowed.prev.clone(), borrowed.next.clone())
						};

						match prev {
							Some(ref mut t) => {
								t.borrow_mut().next = next.clone();
							}
							None => {}
						};

						match next {
							Some(ref mut t) => {
								t.borrow_mut().prev = prev.clone();
							}
							None => {}
						};

						break;
					}

					next_curr = curr_task.borrow().next.clone();
				}
			}

			curr = next_curr.clone();
		}

		let new_head = match self.queues[i].head {
			Some(ref curr_task) => {
				if Rc::ptr_eq(&curr_task, &task) {
					true
				} else {
					false
				}
			}
			None => false,
		};

		if new_head == true {
			self.queues[i].head = task.borrow().next.clone();

			if self.queues[i].head.is_none() {
				self.prio_bitmap &= !(1 << i as u64);
			}
		}
	}
}

pub trait Stack {
	fn top(&self) -> usize;
	fn bottom(&self) -> usize;
}

#[derive(Copy, Clone)]
#[repr(align(64))]
#[repr(C)]
pub struct TaskStack {
	buffer: [u8; STACK_SIZE],
}

impl TaskStack {
	pub const fn new() -> TaskStack {
		TaskStack {
			buffer: [0; STACK_SIZE],
		}
	}
}

impl Stack for TaskStack {
	fn top(&self) -> usize {
		(&(self.buffer[STACK_SIZE - 16]) as *const _) as usize
	}

	fn bottom(&self) -> usize {
		(&(self.buffer[0]) as *const _) as usize
	}
}

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub struct Task {
	/// The ID of this context
	pub id: TaskId,
	/// Task Priority
	pub prio: TaskPriority,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: usize,
	// Stack of the task
	pub stack: Box<dyn Stack>,
	// Physical address of the 1st level page table
	pub root_page_table: usize,
	// next task in queue
	pub next: Option<Rc<RefCell<Task>>>,
	// previous task in queue
	pub prev: Option<Rc<RefCell<Task>>>,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id: id,
			prio: LOW_PRIORITY,
			status: TaskStatus::TaskIdle,
			last_stack_pointer: 0,
			stack: Box::new(crate::arch::mm::get_boot_stack()),
			root_page_table: arch::get_kernel_root_page_table(),
			next: None,
			prev: None,
		}
	}

	pub fn new(id: TaskId, status: TaskStatus, prio: TaskPriority) -> Task {
		Task {
			id: id,
			prio: prio,
			status: status,
			last_stack_pointer: 0,
			stack: Box::new(TaskStack::new()),
			root_page_table: arch::get_kernel_root_page_table(),
			next: None,
			prev: None,
		}
	}
}

pub trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern "C" fn());
}

impl Drop for Task {
	fn drop(&mut self) {
		if self.root_page_table != arch::get_kernel_root_page_table() {
			debug!(
				"Deallocate page table 0x{:x} of task {}",
				self.root_page_table, self.id
			);
			arch::mm::physicalmem::deallocate(self.root_page_table, BasePageSize::SIZE);
		}
	}
}
