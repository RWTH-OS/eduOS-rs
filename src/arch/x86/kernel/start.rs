use crate::arch;
use crate::arch::x86::kernel::processor::shutdown;

extern "C" {
	pub fn main() -> i32;
}

#[cfg(target_arch = "x86")]
extern "C" {
	static mut __bss_start: u8;
	static __bss_end: u8;
}

/// initialize bss section
#[cfg(target_arch = "x86")]
unsafe fn bss_init() {
	core::ptr::write_bytes(
		core::ptr::addr_of_mut!(__bss_start),
		0,
		core::ptr::addr_of!(__bss_end) as usize - core::ptr::addr_of!(__bss_start) as usize,
	);
}

#[cfg(not(test))]
unsafe extern "C" fn entry() -> ! {
	arch::init();

	#[cfg(target_arch = "x86")]
	bss_init();

	let ret = main();

	shutdown(ret)
}

#[cfg(not(test))]
#[cfg(target_arch = "x86_64")]
#[no_mangle]
/// # Safety
///
/// This function is the entry point of the kernel.
/// The kernel itself should not call this function.
pub unsafe extern "C" fn _start() -> ! {
	entry();
}

#[cfg(not(test))]
#[cfg(target_arch = "x86")]
#[no_mangle]
#[naked]
/// # Safety
///
/// This function is the entry point of the kernel.
/// The kernel itself should not call this function.
pub unsafe extern "C" fn _start() -> ! {
	use crate::arch::x86::mm::{BOOT_STACK, BOOT_STACK_SIZE};
	use core::arch::naked_asm;

	naked_asm!(
		"lea esp, {stack}",
		"add esp, {offset}",
		"jmp {entry}",
		stack = sym BOOT_STACK,
		offset = const BOOT_STACK_SIZE - 16,
		entry = sym entry,
		options(noreturn)
	);
}
