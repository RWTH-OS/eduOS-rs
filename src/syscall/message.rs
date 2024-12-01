use crate::scheduler;

#[no_mangle]
pub extern "C" fn sys_message() {
	println!("hello from user task {}!", scheduler::get_current_taskid());
}
