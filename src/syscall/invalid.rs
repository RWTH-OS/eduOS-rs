use crate::logging::*;
use crate::scheduler::*;
use core::arch::naked_asm;

extern "C" fn invalid_syscall(sys_no: u64) -> ! {
	error!("Invalid syscall {}", sys_no);
	do_exit();
}

#[allow(unused_assignments)]
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn sys_invalid() {
	naked_asm!("mov rdi, rax",
		"call {}",
		sym invalid_syscall,
	);
}
