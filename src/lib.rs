#![feature(allocator_api)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

// These need to be visible to the linker, so we need to export them.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use arch::processor::*;
pub use logging::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod console;
