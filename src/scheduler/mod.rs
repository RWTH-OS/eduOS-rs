//! Interface to the scheduler

mod scheduler;
/// task control block
pub mod task;

use crate::errno::*;
use crate::scheduler::task::TaskPriority;

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

/// Initialize module, must be called once, and only once
pub(crate) fn init() {
	unsafe {
		SCHEDULER = Some(scheduler::Scheduler::new());
	}
}

/// Create a new kernel task
pub fn spawn(func: extern "C" fn(), prio: TaskPriority) -> Result<task::TaskId> {
	unsafe { SCHEDULER.as_mut().unwrap().spawn(func, prio) }
}

/// Trigger the scheduler to switch to the next available task
pub fn reschedule() {
	unsafe { SCHEDULER.as_mut().unwrap().reschedule() }
}

/// Terminate the current running task
pub fn do_exit() -> ! {
	unsafe {
		SCHEDULER.as_mut().unwrap().exit();
	}
}

/// Get the TaskID of the current running task
pub fn get_current_taskid() -> task::TaskId {
	unsafe { SCHEDULER.as_ref().unwrap().get_current_taskid() }
}
