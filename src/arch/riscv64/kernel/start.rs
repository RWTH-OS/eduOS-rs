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
	let ret = unsafe { main() };

	shutdown(ret)
}
