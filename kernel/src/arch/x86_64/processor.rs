#![allow(dead_code)]

use cpuio;
use x86::shared::control_regs::*;

pub fn halt() {
	unsafe {
		asm!("hlt" :::: "volatile");
	}
}

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	// shutdown, works like Qemu's shutdown command
	unsafe {
		let mut shutdown_port : cpuio::Port<u8> = cpuio::Port::new(0xf4);
		shutdown_port.write(0x00);
	};

	loop {
		halt();
	}
}

pub fn cpu_init() {
	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | CR0_ALIGNMENT_MASK;
	cr0 = cr0 | CR0_NUMERIC_ERROR;
	cr0 = cr0 | CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(CR0_CACHE_DISABLE|CR0_NOT_WRITE_THROUGH);

	unsafe { cr0_write(cr0) };
}
