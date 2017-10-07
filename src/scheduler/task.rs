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

/// A task control block, which identifies either a process or a thread
#[derive(Debug, Copy, Clone)]
#[repr(align(64))]
pub struct Task {
	/// The ID of this context
    pub id: TaskId,
	/// Status of a task, e.g. if the task is ready or blocked
	pub status: TaskStatus,
	/// Last stack pointer before a context switch to another task
	pub last_stack_pointer: u64,
	/// Stack of the task
	pub stack: KernelStack,
}

pub trait TaskFrame {
    /// Create the initial stack frame for a new task
    fn create_stack_frame(&mut self, func: extern fn());
}

impl Task {
	pub const fn new() -> Task {
		Task {
			id: TaskId::from(0),
			status: TaskStatus::TaskInvalid,
			last_stack_pointer: 0,
			stack: KernelStack::new()
		}
	}
}
