use core::arch::asm;
use core::ptr::write_volatile;

/// MMIO address of QEMU's SiFive test finisher on the `virt` machine.
const SIFIVE_TEST: *mut u32 = 0x0010_0000 as *mut u32;
/// Finisher command that makes QEMU exit successfully.
const FINISHER_PASS: u32 = 0x5555;
/// Finisher command that makes QEMU exit with a failure code.
const FINISHER_FAIL: u32 = 0x3333;

/// The halt function stops the processor until the next interrupt arrives
pub(crate) fn halt() {
	unsafe {
		asm!("wfi", options(nostack, nomem),);
	}
}

/// Shutdown the system
///
/// QEMU's `virt` machine exits when the SiFive test finisher is written: a value
/// of `0x5555` exits successfully, while `0x3333 | (code << 16)` exits with the
/// given error code.
#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
	let value = if error_code == 0 {
		FINISHER_PASS
	} else {
		FINISHER_FAIL | ((error_code as u32) << 16)
	};

	unsafe {
		write_volatile(SIFIVE_TEST, value);
	}

	// The write above terminates QEMU; loop just in case it does not.
	loop {
		halt();
	}
}
