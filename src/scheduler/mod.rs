// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

//! Interface to the scheduler

/// task control block
pub mod task;
mod scheduler;

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

/// Initialite module, must be called once, and only once
pub fn init() {
	unsafe {
		SCHEDULER = Some(scheduler::Scheduler::new());
	}
}

/// Create a new kernel task
pub fn spawn(func: extern fn()) -> task::TaskId {
	unsafe {
		SCHEDULER.as_mut().unwrap().spawn(func)
	}
}

/// Trigger the scheduler to switch to the next available task
pub fn reschedule() {
	unsafe {
		SCHEDULER.as_mut().unwrap().reschedule()
	}
}

/// Terminate the current running task
pub fn do_exit() {
	unsafe {
		SCHEDULER.as_mut().unwrap().exit();
	}
}

/// Get the TaskID of the current running task
pub fn get_current_taskid() -> task::TaskId {
	unsafe {
		SCHEDULER.as_ref().unwrap().get_current_taskid()
	}
}
