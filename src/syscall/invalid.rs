use crate::logging::*;
use crate::scheduler::*;
use core::arch::asm;

extern "C" fn invalid_syscall(sys_no: u64) -> ! {
	error!("Invalid syscall {}", sys_no);
	do_exit();
}

#[allow(unused_assignments)]
#[naked]
pub(crate) unsafe extern "C" fn sys_invalid() {
	asm!("mov rdi, rax",
		"call {}",
		sym invalid_syscall,
		options(noreturn)
	);
}
