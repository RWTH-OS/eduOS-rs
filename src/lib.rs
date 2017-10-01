#![feature(asm, const_fn, lang_items)]
#![no_std]

extern crate cpuio;
extern crate rlibc;
extern crate spin;
extern crate x86;

use core::fmt::Write;

// These need to be visible to the linker, so we need to export them.
pub use runtime_glue::*;

#[macro_use]
mod macros;
mod runtime_glue;
mod arch;
mod console;

#[no_mangle]
pub extern "C" fn rust_main() {
    println!("Hello, world!");

    loop {}
}
