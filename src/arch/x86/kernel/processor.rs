#![allow(dead_code)]

use crate::logging::*;
use core::arch::asm;
#[cfg(feature = "qemu-exit")]
use qemu_exit::QEMUExit;
use x86::controlregs::*;

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub fn mb() {
	unsafe {
		asm!("mfence", options(preserves_flags, nostack));
	}
}

/// Search the most significant bit
#[inline(always)]
pub(crate) fn msb(value: u64) -> Option<u64> {
	if value > 0 {
		let ret: u64;

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
#[inline(always)]
pub(crate) fn lsb(value: u64) -> Option<u64> {
	if value > 0 {
		let ret: u64;
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

pub(crate) fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

pub(crate) fn pause() {
	unsafe {
		asm!("pause", options(nomem, nostack));
	}
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn shutdown(error_code: i32) -> ! {
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

pub fn init() {
	debug!("enable supported processor features");

	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | Cr0::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | Cr0::CR0_NUMERIC_ERROR;
	cr0 = cr0 | Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { cr0_write(cr0) };
}
