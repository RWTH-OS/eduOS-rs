#![allow(dead_code)]

//! Configuration parameter of the kernel eduOS-rs

use crate::arch::mm::VirtAddr;

/// Define the size of the kernel stack
pub const STACK_SIZE: usize = 0x4000;

/// Size of a cache line
pub const CACHE_LINE: usize = 64;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 32;

/// frequency of the timer interrupt
pub const TIMER_FREQ: u32 = 100; /* in HZ */

/// Entry point of the user tasks
pub const USER_ENTRY: VirtAddr = VirtAddr(0x20000000000u64);

/// Size of the kernel heap
pub const HEAP_SIZE: usize = 8 * 1024 * 1024;
