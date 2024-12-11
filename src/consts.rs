#![allow(dead_code)]

//! Configuration parameter of the kernel eduOS-rs

use crate::arch::mm::VirtAddr;

/// Define the size of the kernel stack
pub(crate) const STACK_SIZE: usize = 0x3000;

/// Define the size of the interrupt stack
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) const INTERRUPT_STACK_SIZE: usize = 0x3000;

/// Size of a cache line
pub(crate) const CACHE_LINE: usize = 64;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 32;

/// frequency of the timer interrupt
pub(crate) const TIMER_FREQ: u32 = 100; /* in HZ */

/// Entry point of the user tasks
pub const USER_ENTRY: VirtAddr = VirtAddr(0x20000000000u64);

/// Size of the kernel heap
pub(crate) const HEAP_SIZE: usize = 8 * 1024 * 1024;
