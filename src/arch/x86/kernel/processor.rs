use crate::logging::*;
use core::arch::asm;
#[cfg(feature = "qemu-exit")]
use qemu_exit::QEMUExit;
use x86::controlregs::*;

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub(crate) fn mb() {
	unsafe {
		asm!("mfence", options(preserves_flags, nostack));
	}
}

/// Search the most significant bit
#[inline(always)]
pub(crate) fn msb(value: usize) -> Option<usize> {
	if value > 0 {
		let ret: usize;

		unsafe {
			asm!("bsr {0}, {1}",
				out(reg) ret,
				in(reg) value,
				options(nomem, nostack)
			);
		}
		Some(ret)
	} else {
		None
	}
}

/// Search the least significant bit
#[allow(dead_code)]
#[inline(always)]
pub(crate) fn lsb(value: usize) -> Option<usize> {
	if value > 0 {
		let ret: usize;

		unsafe {
			asm!("bsf {0}, {1}",
				out(reg) ret,
				in(reg) value,
				options(nomem, nostack)
			);
		}
		Some(ret)
	} else {
		None
	}
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

#[inline(always)]
pub(crate) fn pause() {
	unsafe {
		asm!("pause", options(nomem, nostack));
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
		halt();
	}
}

pub(crate) fn init() {
	debug!("enable supported processor features");

	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 |= Cr0::CR0_ALIGNMENT_MASK;
	cr0 |= Cr0::CR0_NUMERIC_ERROR;
	cr0 |= Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 &= !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { cr0_write(cr0) };
}
