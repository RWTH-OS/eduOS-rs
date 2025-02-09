use crate::syscall::SYSHANDLER_TABLE;
use core::arch::naked_asm;

#[naked]
pub(crate) unsafe extern "C" fn syscall_handler() {
	naked_asm!(
		// save context, see x86_64 ABI
		"push rcx",
		"push rdx",
		"push rsi",
		"push rdi",
		"push r8",
		"push r9",
		"push r10",
		"push r11",
		// switch to kernel stack
		"swapgs",
		"mov rcx, rsp",
		"rdgsbase rsp",
		"push rcx",
		// copy 4th argument to rcx to adhere x86_64 ABI
		"mov rcx, r10",
		"sti",
		"call [{sys_handler}+8*rax]",
		// restore context, see x86_64 ABI
		"cli",
		// switch to user stack
		"pop rcx",
		"mov rsp, rcx",
		"swapgs",
		"pop r11",
		"pop r10",
		"pop r9",
		"pop r8",
		"pop rdi",
		"pop rsi",
		"pop rdx",
		"pop rcx",
		"sysretq",
		sys_handler = sym SYSHANDLER_TABLE,
		options(noreturn));
}
