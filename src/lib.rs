#![feature(asm, const_fn, lang_items)]
#![no_std]

extern crate cpuio;
extern crate rlibc;
extern crate spin;
extern crate x86;

// These need to be visible to the linker, so we need to export them.
pub use runtime_glue::*;
pub use logging::*;

#[macro_use]
mod macros;
#[macro_use]
mod logging;
mod runtime_glue;
mod arch;
mod console;

#[no_mangle]
pub extern "C" fn rust_main() {
    println!("Hello, world!");

    loop {}
}
