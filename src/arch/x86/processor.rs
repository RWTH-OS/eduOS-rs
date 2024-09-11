use x86::controlregs::*;
#[cfg(feature = "qemu_exit")]
use x86::io;

pub(crate) fn halt() {
	unsafe {
		x86::halt();
	}
}

#[cfg(feature = "qemu_exit")]
fn qemu_exit(success: bool) {
	let code = if success { 3 >> 1 } else { 0 };
	unsafe {
		io::outl(0xf4, code);
	}
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn shutdown(error_code: i32) -> ! {
	#[cfg(feature = "qemu_exit")]
	qemu_exit(error_code == 0);
	loop {
		halt();
	}
}

pub(crate) fn cpu_init() {
	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | Cr0::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | Cr0::CR0_NUMERIC_ERROR;
	cr0 = cr0 | Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	unsafe { cr0_write(cr0) };
}
