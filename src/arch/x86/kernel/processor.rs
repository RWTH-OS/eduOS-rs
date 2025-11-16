use core::arch::asm;
#[cfg(feature = "qemu-exit")]
use qemu_exit::QEMUExit;
use x86::controlregs::*;

pub(crate) fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
	#[cfg(feature = "qemu-exit")]
	{
		let code = if error_code == 0 { 5 } else { 1 };

		// shutdown, works like Qemu's shutdown command
		let qemu_exit_handle = qemu_exit::X86::new(0xf4, code);
		qemu_exit_handle.exit_success();
	}

	#[cfg(not(feature = "qemu-exit"))]
	loop {
		unsafe {
			x86::halt();
		}
	}
}

pub(crate) fn cpu_init() {
	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 |= Cr0::CR0_ALIGNMENT_MASK;
	cr0 |= Cr0::CR0_NUMERIC_ERROR;
	cr0 |= Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 &= !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	unsafe { cr0_write(cr0) };
}
