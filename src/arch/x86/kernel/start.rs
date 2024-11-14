use crate::arch::x86::kernel::processor::shutdown;

extern "C" {
	pub fn main() -> i32;
}

#[cfg(not(test))]
unsafe extern "C" fn entry() -> ! {
	let ret = main();

	shutdown(ret)
}

#[cfg(not(test))]
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
	crate::arch::x86::kernel::BOOT_INFO = Some(boot_info);

	entry();
}

#[cfg(not(test))]
#[cfg(target_arch = "x86")]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() -> ! {
	use crate::arch::mm::{BOOT_STACK, BOOT_STACK_SIZE};
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
