#![allow(dead_code)]
use core::arch::asm;

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	unsafe {
		asm!(
			"mov w0, 0x18",
			"mov x1, #0x20000",
			"add x1, x1, #0x26",
			"hlt #0xF000",
			options(nostack, nomem, noreturn),
		);
	}
}
