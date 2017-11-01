// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#![allow(dead_code)]

use consts::*;
use core;
use alloc;
use alloc::heap::{Heap, Alloc, Layout};
use core::ptr::Shared;
use logging::*;
use arch::processor::lsb;

/// The status of the task - used for scheduling
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
	TaskInvalid,
	TaskReady,
	TaskRunning,
	TaskBlocked,
	TaskFinished,
	TaskIdle
}

/// Unique identifier for a task (i.e. `pid`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(usize);

impl TaskId {
	pub const fn into(self) -> usize {
		self.0
	}

	pub const fn from(x: usize) -> Self {
		TaskId(x)
	}
}

impl alloc::fmt::Display for TaskId {
	fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Priority of a task
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Priority(u8);

impl Priority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		Priority(x)
	}
}

impl alloc::fmt::Display for Priority {
	fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
		write!(f, "{}", self.0)
	}
}

pub const REALTIME_PRIO: Priority = Priority::from(0);
pub const HIGH_PRIO: Priority = Priority::from(0);
pub const NORMAL_PRIO: Priority = Priority::from(24);
pub const LOW_PRIO: Priority = Priority::from(NO_PRIORITIES as u8 - 1);

#[derive(Copy, Clone)]
#[repr(align(64))]
pub struct KernelStack {
	buffer: [u8; KERNEL_STACK_SIZE]
}

impl KernelStack {
	pub const fn new() -> KernelStack {
		KernelStack {
			buffer: [0; KERNEL_STACK_SIZE]
		}
	}

	pub fn top(&self) -> usize {
		(&(self.buffer[KERNEL_STACK_SIZE - 1]) as *const _) as usize
	}

	pub fn bottom(&self) -> usize {
		(&(self.buffer[0]) as *const _) as usize
	}
}

/// The stack is too large to use the default debug trait. => create our own.
impl core::fmt::Debug for KernelStack {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		for x in self.buffer.iter() {
			write!(f, "{:X}", x)?;
		}

		Ok(())
	}
}

/// Simple task queue, which doesn't need any allocation of memory
#[derive(Copy, Clone)]
pub struct TaskQueue {
	head: Option<Shared<Task>>,
	tail: Option<Shared<Task>>
}

impl TaskQueue {
	/// Creates an empty task queue
	pub const fn new() -> TaskQueue {
		TaskQueue {
			head: None,
			tail: None
		}
	}

	/// Check if the queue is empty
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		match self.head {
			None => true,
			Some(_h) => false
		}
	}

	/// Add task at the end of the queue
	pub fn push_back(&mut self, task: &mut Shared<Task>) {
		unsafe {
			match self.tail {
				None => {
					task.as_mut().prev = None;
					task.as_mut().next = None;
					self.head = Some(*task)
				},
				Some(mut tail) => {
					task.as_mut().prev = Some(tail);
					task.as_mut().next = None;
					tail.as_mut().next = Some(*task);
				}
			}

			self.tail = Some(*task);
		}
	}

	/// Pop the first task of the queue
	pub fn pop_front(&mut self) -> Option<Shared<Task>> {
		unsafe {
			match self.head {
				None => None,
				Some(mut task) => {
					self.head = task.as_mut().next;
					// is the queue empty? => set tail to None
					match self.head {
						None => self.tail = None,
						Some(_i) => {}
					}
					Some(task)
				}
			}
		}
	}
}

/// Realize a priority queue for tasks
pub struct PriorityTaskQueue {
	queues: [TaskQueue; NO_PRIORITIES],
	prio_bitmap: u64
}

impl PriorityTaskQueue {
	/// Creates an empty priority queue for tasks
	pub const fn new() -> PriorityTaskQueue {
		PriorityTaskQueue {
			queues: [TaskQueue::new(); NO_PRIORITIES],
			prio_bitmap: 0
		}
	}

	/// Add task by its priority to the queue
	pub fn push(&mut self, prio: Priority, task: &mut Shared<Task>) {
		let mut i = prio.into() as usize;

		if i >= NO_PRIORITIES {
			info!("priority with {} is too high for TaskQueue::push_back()!", prio);
			i = NO_PRIORITIES - 1;
		}

		self.prio_bitmap |= 1 << i;
		self.queues[i].push_back(task);
	}

	/// Pop the task with the highest priority from the queue
	pub fn pop(&mut self) -> Option<Shared<Task>> {
		let i = lsb(self.prio_bitmap);

		if i < NO_PRIORITIES as u64 {
			let ret = self.queues[i as usize].pop_front();

			if self.queues[i as usize].is_empty() == true {
				self.prio_bitmap &= !(1 << i);
			}

			ret
		} else {
			None
		}
	}

	/// Pop the next task, which has a higher or the same proority like `prio`
	pub fn pop_with_prio(&mut self, prio: Priority) -> Option<Shared<Task>> {
		let i = lsb(self.prio_bitmap);

		if i <= prio.into() as u64 {
			let ret = self.queues[i as usize].pop_front();

			if self.queues[i as usize].is_empty() == true {
				self.prio_bitmap &= !(1 << i);
			}

			ret
		} else {
			None
		}
	}
}

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub struct Task {
	/// The ID of this context
	pub id: TaskId,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Task priority,
	pub prio: Priority,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: u64,
	/// points to the next task within a task queue
	next: Option<Shared<Task>>,
	/// points to the previous task within a task queue
	prev: Option<Shared<Task>>,
	/// Stack of the task
	pub stack: *mut KernelStack,
}

pub trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern fn());
}

impl Drop for Task {
    fn drop(&mut self) {
		debug!("deallocate stack of task {} (stack at 0x{:x})", self.id, self.stack as usize);

		// deallocate stack
		unsafe {
			Heap.dealloc(self.stack as *mut u8, Layout::new::<KernelStack>());
		}
	}
}

impl Task {
	pub fn new(tid: TaskId, task_status: TaskStatus, task_prio: Priority) -> Task {
		let tmp = unsafe { Heap.alloc(Layout::new::<KernelStack>()).unwrap() as *mut KernelStack };

		debug!("allocate stack for task {} at 0x{:x}", tid, tmp as usize);

		Task {
			id: tid,
			status: task_status,
			prio: task_prio,
			last_stack_pointer: 0,
			next: None,
			prev: None,
			// allocate stack directly from the heap
			stack: tmp
		}
	}
}
