// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Implementation of the Allocator for dynamically allocating heap memory
//! in the kernel.
//!
//! The data structures used to manage heap memory require dynamic memory allocations
//! themselves. To solve this chicken-egg problem, eduOS-rs first uses a
//! "Bootstrap Allocator". This is a simple single-threaded implementation using some
//! preallocated space, along with an index variable.
//! Freed memory is never reused, but this can be neglected for bootstrapping.

use crate::logging::*;
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;

/// Size of the preallocated space for the Bootstrap Allocator.
const BOOTSTRAP_HEAP_SIZE: usize = 2 * 1024 * 1024;

/// The Allocator structure is immutable, so we need this helper structure
/// for our allocator information.
#[repr(align(64))]
#[repr(C)]
struct AllocatorInfo {
	heap: [u8; BOOTSTRAP_HEAP_SIZE],
	index: usize,
}

impl AllocatorInfo {
	const fn new() -> AllocatorInfo {
		AllocatorInfo {
			heap: [0x00; BOOTSTRAP_HEAP_SIZE],
			index: 0,
		}
	}
}

static mut ALLOCATOR_INFO: AllocatorInfo = AllocatorInfo::new();

pub struct Allocator;

unsafe impl<'a> GlobalAlloc for &'a Allocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		alloc_bootstrap(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		// We never deallocate memory of the Bootstrap Allocator.
		debug!("Deallocate {} bytes at {:#X}", layout.size(), ptr as usize);
	}
}

/// An allocation using the always available Bootstrap Allocator.
unsafe fn alloc_bootstrap(layout: Layout) -> *mut u8 {
	let ptr = &mut ALLOCATOR_INFO.heap[align_up!(ALLOCATOR_INFO.index, layout.align())] as *mut u8;
	debug!(
		"Allocating {:#X} bytes at {:#X}, index {}",
		layout.size(),
		ptr as usize,
		ALLOCATOR_INFO.index
	);

	if ALLOCATOR_INFO.index + layout.size() >= BOOTSTRAP_HEAP_SIZE {
		panic!("Bootstrap Allocator Overflow! Increase BOOTSTRAP_HEAP_SIZE.");
	}

	ALLOCATOR_INFO.index = align_up!(ALLOCATOR_INFO.index, layout.align()) + layout.size();

	ptr
}

pub fn init() {}
