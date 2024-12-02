use crate::logging::*;
use crate::scheduler::*;

pub(crate) extern "C" fn sys_exit() {
	debug!("enter syscall exit");
	do_exit();
}
