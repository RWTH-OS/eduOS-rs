#![allow(dead_code)]

use crate::logging::*;
use x86::controlregs::*;
use x86::io::*;

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub fn mb() {
	unsafe {
		llvm_asm!("mfence" ::: "memory" : "volatile");
	}
}

/// Search the most significant bit
#[inline(always)]
pub fn msb(value: u64) -> Option<u64> {
	println!("value {}", value);
	if value > 0 {
		let ret: u64;
		unsafe {
			asm!("bsr {0}, {1}",
			    out(reg) ret,
				in(reg) value,
			    options(nomem, nostack)
			);
		}
		println!("value {} {}", value, ret);
		Some(ret)
	} else {
		None
	}
}

/// Search the least significant bit
#[inline(always)]
pub fn lsb(value: u64) -> Option<u64> {
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

pub fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

pub fn pause() {
	unsafe {
		asm!("pause", options(nomem, nostack));
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
