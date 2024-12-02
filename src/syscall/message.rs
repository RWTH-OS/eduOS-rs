use crate::scheduler;

pub(crate) extern "C" fn sys_message() {
	println!("hello from user task {}!", scheduler::get_current_taskid());
}
