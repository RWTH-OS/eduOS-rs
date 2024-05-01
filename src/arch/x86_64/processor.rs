use x86::controlregs::*;
use x86_64::instructions::port::Port;

pub fn halt() {
	unsafe {
		x86::halt();
	}
}

fn qemu_exit(success: bool) {
	let code = if success { 3 >> 1 } else { 0 };
	unsafe {
		Port::<u32>::new(0xf4).write(code);
	}
}

#[no_mangle]
pub extern "C" fn shutdown(error_code: i32) -> ! {
	qemu_exit(error_code == 0);
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
