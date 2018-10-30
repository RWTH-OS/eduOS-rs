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

use alloc;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt;
use alloc::alloc::{alloc, dealloc, Layout};
use collections::{DoublyLinkedList, Node};
use logging::*;
use consts::*;

extern {
    fn get_bootstack() -> *mut u8;
}

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

#[derive(Copy, Clone)]
#[repr(align(64))]
#[repr(C)]
pub struct Stack {
	buffer: [u8; STACK_SIZE]
}

impl Stack {
	pub const fn new() -> Stack {
		Stack {
			buffer: [0; STACK_SIZE]
		}
	}

	pub fn top(&self) -> usize {
		(&(self.buffer[STACK_SIZE - 16]) as *const _) as usize
	}

	pub fn bottom(&self) -> usize {
		(&(self.buffer[0]) as *const _) as usize
	}
}

pub struct TaskQueue {
	queue: DoublyLinkedList<Rc<RefCell<Task>>>
}

impl TaskQueue {
	pub fn new() -> TaskQueue {
		TaskQueue {
			queue: Default::default()
		}
	}

	/// Add a task by its priority to the queue
	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		self.queue.push(Node::new(task));
	}

	/// Pop the task from the queue
	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		let first_task = self.queue.head();
		first_task.map(|task| {
			self.queue.remove(task.clone());

			task.borrow().value.clone()
		})
	}

	/// Remove a specific task from the priority queue.
	pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
		for node in self.queue.iter() {
			if Rc::ptr_eq(&node.borrow().value, &task) {
				self.queue.remove(node.clone());

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
	pub stack: *mut Stack,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		let stack = unsafe { alloc(Layout::new::<Stack>()) as *mut Stack };

		Task {
			id: id,
			status: TaskStatus::TaskIdle,
			last_stack_pointer: 0,
			stack: stack
		}
	}

	pub fn new(id: TaskId, status: TaskStatus) -> Task {
		let stack = unsafe { alloc(Layout::new::<Stack>()) as *mut Stack };

		debug!("Allocate stack for task {} at 0x{:x}", id, stack as usize);

		Task {
			id: id,
			status: status,
			last_stack_pointer: 0,
			stack: stack
		}
	}
}

pub trait TaskFrame {
	/// Create the initial stack frame for a new task
	fn create_stack_frame(&mut self, func: extern fn());
}

impl Drop for Task {
	fn drop(&mut self) {
		debug!("Deallocate stack of task {} (stack at 0x{:x})", self.id, self.stack as usize);

		// deallocate stack
		unsafe { dealloc(self.stack as *mut u8, Layout::new::<Stack>()); }
	}
}
