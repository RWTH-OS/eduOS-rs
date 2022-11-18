// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use crate::consts::*;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
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

pub struct TaskQueue {
	queue: LinkedList<Rc<RefCell<Task>>>,
}

impl TaskQueue {
	pub fn new() -> TaskQueue {
		TaskQueue {
			queue: Default::default(),
		}
	}

	/// Add a task to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		self.queue.push_back(task);
	}

	/// Pop the task from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		self.queue.pop_front()
	}

	/// Remove a specific task from the priority queue.
	pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
		let mut cursor = self.queue.cursor_front_mut();

		// Loop through all blocked tasks to find it.
		while let Some(node) = cursor.current() {
			if Rc::ptr_eq(&node, &task) {
				// Remove it from the list
				cursor.remove_current();

				break;
			}
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
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: usize,
	// Stack of the task
	pub stack: Box<dyn Stack>,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id: id,
			status: TaskStatus::TaskIdle,
			last_stack_pointer: 0,
			stack: Box::new(crate::arch::mm::get_boot_stack()),
		}
	}

	pub fn new(id: TaskId, status: TaskStatus) -> Task {
		Task {
			id: id,
			status: status,
			last_stack_pointer: 0,
			stack: Box::new(TaskStack::new()),
		}
	}
}

pub trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern "C" fn());
}
