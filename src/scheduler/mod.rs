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
#![allow(private_no_mangle_fns)]

//! Interface to the scheduler

use core::ptr::Shared;

/// task control block
pub mod task;
mod scheduler;

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

/// Initialite module, must be called once, and only once
pub fn init() {
	unsafe {
		SCHEDULER = Some(scheduler::Scheduler::new());
		debug!("scheduler initialized");
	}
}

/// Create a new kernel task
#[inline(always)]
pub fn spawn(func: extern fn(), prio: task::Priority) -> task::TaskId {
	unsafe { SCHEDULER.as_mut().unwrap().spawn(func, prio) }
}

/// Trigger the scheduler to switch to the next available task
#[inline(always)]
pub fn reschedule() {
	unsafe {
		SCHEDULER.as_mut().unwrap().reschedule()
	}
}

#[inline(always)]
pub fn number_of_tasks() -> usize {
	unsafe {
		SCHEDULER.as_mut().unwrap().number_of_tasks()
	}
}

/// Trigger the scheduler from an interrupt to switch to the next available task
#[inline(always)]
pub fn schedule() {
	unsafe {
		SCHEDULER.as_mut().unwrap().schedule()
	}
}

/// Set current task status to TaskBlocked
#[inline(always)]
pub fn block_current_task() -> Shared<task::Task> {
	unsafe {
		SCHEDULER.as_mut().unwrap().block_current_task()
	}
}

#[inline(always)]
pub fn get_current_stack() -> (usize, usize) {
	unsafe {
		SCHEDULER.as_mut().unwrap().get_current_stack()
	}
}

#[inline(always)]
pub fn wakeup_task(task: Shared<task::Task>) {
	unsafe {
		SCHEDULER.as_mut().unwrap().wakeup_task(task)
	}
}

/// Terminate the current running task
#[inline(always)]
pub fn exit() -> ! {
	unsafe {
		SCHEDULER.as_mut().unwrap().exit()
	}
}

/// Terminate the current running task
#[inline(always)]
pub fn abort() -> ! {
	unsafe {
		SCHEDULER.as_mut().unwrap().abort()
	}
}

/// Get the TaskID of the current running task
#[inline(always)]
pub fn get_current_taskid() -> task::TaskId {
	unsafe {
		SCHEDULER.as_mut().unwrap().get_current_taskid()
	}
}

#[inline(always)]
pub fn get_current_priority() -> task::Priority {
	unsafe {
		SCHEDULER.as_mut().unwrap().get_current_priority()
	}
}

/// Get prioritiy of task with Identifier tid
#[inline(always)]
pub fn get_priority(id: task::TaskId) -> task::Priority {
	unsafe {
		SCHEDULER.as_mut().unwrap().get_priority(id)
	}
}
