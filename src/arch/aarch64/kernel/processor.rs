use core::arch::asm;

use semihosting::process::exit;

/// The halt function stops the processor until the next interrupt arrives
#[allow(dead_code)]
pub(crate) fn halt() {
	unsafe {
		asm!("wfi", options(nostack, nomem),);
	}
}

/// Shutdown the system
#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
	exit(error_code)
}
