#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

/// This function isn't the entry point, since the linker looks for a function
/// named `_start` by default. But `_start` jumps directly to `main`
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> i32 {
	eduos_rs::arch::init();

	println!("Hello world!");

	0
}
