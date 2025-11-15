#![allow(dead_code)]

//! Configuration parameter of the kernel eduOS-rs

/// Define the size of the kernel stack
#[cfg(target_arch = "x86_64")]
pub(crate) const STACK_SIZE: usize = 0x3000;

/// Define the size of the kernel stack
#[cfg(target_arch = "x86")]
pub(crate) const STACK_SIZE: usize = 0x2000;

/// Define the size of the kernel stack
#[cfg(target_arch = "aarch64")]
pub(crate) const STACK_SIZE: usize = 0x3000;

/// Size of a cache line
pub(crate) const CACHE_LINE: usize = 64;

/// Size of a page frame on a x86_64 processor
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) const PAGE_SIZE: usize = 4096;

/// Size of the kernel heap
pub(crate) const HEAP_SIZE: usize = 8 * 1024 * 1024;
