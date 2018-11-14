#![feature(asm, const_fn, lang_items)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(integer_atomics)]
#![feature(compiler_builtins_lib)]
#![feature(naked_functions)]
#![no_std]

extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate cpuio;
#[cfg(target_arch = "x86_64")]
extern crate x86;
extern crate alloc;

// These need to be visible to the linker, so we need to export them.
pub use logging::*;
#[cfg(target_arch = "x86_64")]
pub use arch::processor::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod consts;
pub mod arch;
pub mod console;
pub mod mm;
pub mod collections;
pub mod scheduler;
pub mod errno;
pub mod synch;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;
