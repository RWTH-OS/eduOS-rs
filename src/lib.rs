#![feature(asm, const_fn, lang_items)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![no_std]

extern crate alloc;
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
pub mod collections;
pub mod console;
pub mod consts;
pub mod errno;
pub mod mm;
pub mod scheduler;
pub mod synch;
pub mod syscall;
pub mod rlib;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;
