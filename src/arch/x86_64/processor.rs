#![allow(dead_code)]

use x86::controlregs::*;
use x86::io::*;

pub fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	// shutdown, works like Qemu's shutdown command
	unsafe {
		outb(0xf4, 0x00);
	}

	loop {
		halt();
	}
}

pub fn cpu_init() {
	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | Cr0::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | Cr0::CR0_NUMERIC_ERROR;
	cr0 = cr0 | Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	unsafe { cr0_write(cr0) };
}
