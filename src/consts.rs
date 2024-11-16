#![allow(dead_code)]

//! Configuration parameter of the kernel eduOS-rs

/// Define the size of the kernel stack
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) const STACK_SIZE: usize = 0x3000;

/// Size of a cache line
pub(crate) const CACHE_LINE: usize = 64;

/// Size of a page frame on a x86_64 processor
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) const PAGE_SIZE: usize = 4096;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 32;

/// frequency of the timer interrupt
pub(crate) const TIMER_FREQ: u32 = 100; /* in HZ */

/// Size of the kernel heap
pub(crate) const HEAP_SIZE: usize = 8 * 1024 * 1024;
