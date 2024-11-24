//! Interface to the scheduler

mod scheduler;
/// task control block
pub mod task;

use crate::arch;
use crate::arch::mm::{PhysAddr, VirtAddr};
use crate::errno::*;
use crate::scheduler::task::{Task, TaskPriority};
use alloc::rc::Rc;
use core::cell::RefCell;

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

/// Initialize module, must be called once, and only once
pub(crate) fn init() {
	unsafe {
		SCHEDULER = Some(scheduler::Scheduler::new());
	}

	arch::register_task();
}

/// Create a new kernel task
pub fn spawn(func: extern "C" fn(), prio: TaskPriority) -> Result<task::TaskId> {
	unsafe { SCHEDULER.as_mut().unwrap().spawn(func, prio) }
}

/// Trigger the scheduler to switch to the next available task
pub fn reschedule() {
	unsafe { SCHEDULER.as_mut().unwrap().reschedule() }
}

/// Timer interrupt  call scheduler to switch to the next available task
pub(crate) fn schedule() {
	unsafe { SCHEDULER.as_mut().unwrap().schedule() }
}

/// Terminate the current running task
pub fn do_exit() -> ! {
	unsafe {
		SCHEDULER.as_mut().unwrap().exit();
	}
}

/// Terminate the current running task
pub fn abort() -> ! {
	unsafe { SCHEDULER.as_mut().unwrap().abort() }
}

pub(crate) fn get_current_stack() -> VirtAddr {
	unsafe { SCHEDULER.as_mut().unwrap().get_current_stack() }
}

pub(crate) fn get_root_page_table() -> PhysAddr {
	unsafe { SCHEDULER.as_mut().unwrap().get_root_page_table() }
}

pub(crate) fn set_root_page_table(addr: PhysAddr) {
	unsafe {
		SCHEDULER.as_mut().unwrap().set_root_page_table(addr);
	}
}

pub(crate) fn block_current_task() -> Rc<RefCell<Task>> {
	unsafe { SCHEDULER.as_mut().unwrap().block_current_task() }
}

pub(crate) fn wakeup_task(task: Rc<RefCell<Task>>) {
	unsafe { SCHEDULER.as_mut().unwrap().wakeup_task(task) }
}

/// Get the TaskID of the current running task
pub fn get_current_taskid() -> task::TaskId {
	unsafe { SCHEDULER.as_ref().unwrap().get_current_taskid() }
}

pub(crate) struct DisabledPreemption {
	irq_enabled: bool,
}

impl DisabledPreemption {
	pub fn new() -> Self {
		DisabledPreemption {
			irq_enabled: arch::irq::irq_nested_disable(),
		}
	}
}

impl Drop for DisabledPreemption {
	fn drop(&mut self) {
		arch::irq::irq_nested_enable(self.irq_enabled);
	}
}
