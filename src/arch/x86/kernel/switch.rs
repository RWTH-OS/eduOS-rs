use crate::arch::mm::VirtAddr;
use crate::arch::x86::kernel::gdt::set_current_kernel_stack;
use core::arch::naked_asm;

#[cfg(target_arch = "x86_64")]
macro_rules! save_context {
	() => {
		concat!(
			r#"
			pushfq
			push rax
			push rcx
			push rdx
			push rbx
			sub  rsp, 8
			push rbp
			push rsi
			push rdi
			push r8
			push r9
			push r10
			push r11
			push r12
			push r13
			push r14
			push r15
			"#,
		)
	};
}

#[cfg(target_arch = "x86_64")]
macro_rules! restore_context {
	() => {
		concat!(
			r#"
			pop r15
			pop r14
			pop r13
			pop r12
			pop r11
			pop r10
			pop r9
			pop r8
			pop rdi
			pop rsi
			pop rbp
			add rsp, 8
			pop rbx
			pop rdx
			pop rcx
			pop rax
			popfq
			ret
			"#
		)
	};
}

#[cfg(target_arch = "x86_64")]
#[naked]
/// # Safety
///
/// Only the scheduler itself should call this function to switch the
/// context. `old_stack` is a pointer, where the address to the old
/// stack is stored. `new_stack` provides the stack pointer of the
/// next task.
pub(crate) unsafe extern "C" fn switch(_old_stack: *mut VirtAddr, _new_stack: VirtAddr) {
	// rdi = old_stack => the address to store the old rsp
	// rsi = new_stack => stack pointer of the new task

	naked_asm!(
		save_context!(),
		"rdfsbase rax",
		"rdgsbase rdx",
		"push rax",
		"push rdx",
		// Store the old `rsp` behind `old_stack`
		"mov [rdi], rsp",
		// Set `rsp` to `new_stack`
		"mov rsp, rsi",
		// Set task switched flag
		"mov rax, cr0",
		"or rax, 8",
		"mov cr0, rax",
		// set stack pointer in TSS
		"call {set_stack}",
		"pop r15",
		"wrgsbase r15",
		"pop r15",
		"wrfsbase r15",
		restore_context!(),
		set_stack = sym set_current_kernel_stack,
		options(noreturn)
	);
}

#[cfg(target_arch = "x86")]
#[naked]
/// # Safety
///
/// Only the scheduler itself should call this function to switch the
/// context. `old_stack` is a pointer, where the address to the old
/// stack is stored. `new_stack` provides the stack pointer of the
/// next task.
pub unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
	naked_asm!(
		// store all registers
		"pushfd",
		"pushad",
		// switch stack
		"mov edi, [esp+10*4]",
		"mov [edi], esp",
		"mov esp, [esp+11*4]",
		// restore registers
		"popad",
		"popfd",
		"ret",
		options(noreturn)
	);
}
