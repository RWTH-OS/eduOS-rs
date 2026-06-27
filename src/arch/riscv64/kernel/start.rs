#![allow(dead_code)]

use core::arch::global_asm;

use crate::shutdown;

extern "C" {
	pub fn main() -> i32;
}

global_asm!(
	include_str!("start.s"),
	start_rust = sym start_rust,
);

#[inline(never)]
pub unsafe fn start_rust() -> ! {
	// Install the trap vector and start the periodic timer. Interrupts remain
	// globally masked until `main` enables them explicitly.
	crate::arch::init();

	let ret = unsafe { main() };

	shutdown(ret)
}
