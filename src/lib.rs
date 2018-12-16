#![feature(asm, const_fn, lang_items)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(integer_atomics)]
#![feature(compiler_builtins_lib)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![no_std]

#[cfg(target_arch = "x86_64")]
extern crate cpuio;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[cfg(target_arch = "x86_64")]
extern crate raw_cpuid;
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
pub mod syscall;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;
