#![allow(dead_code)]

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	unsafe {
		asm!("mov w0, 0x18\n\t
			mov x1, #0x20000\n\t
			add x1, x1, #0x26\n\t
			hlt #0xF000" :::: "volatile");
	}

	loop {}
}
