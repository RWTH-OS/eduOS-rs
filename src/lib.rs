#![feature(lang_items)]
#![feature(asm_const)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![no_std]

extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate x86;

// These need to be visible to the linker, so we need to export them.
#[cfg(target_arch = "x86_64")]
pub use arch::processor::*;
pub use logging::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod console;
