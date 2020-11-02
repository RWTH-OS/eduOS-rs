#![feature(llvm_asm, const_fn, lang_items)]
#![feature(linked_list_cursors)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![no_std]

extern crate alloc;
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
pub mod consts;
pub mod errno;
pub mod mm;
pub mod scheduler;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;
