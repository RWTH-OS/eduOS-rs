// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Implementation of the Allocator for dynamically allocating heap memory
//! in the kernel.
//!
//! The data structures used to manage heap memory require dynamic memory allocations
//! themselves. To solve this chicken-egg problem, eduOS-rs first uses a
//! "Bootstrap Allocator". This is a simple single-threaded implementation using some
//! preallocated space, along with an index variable.
//! Freed memory is never reused, but this can be neglected for bootstrapping.

use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use consts::*;
use logging::*;

/// Size of the preallocated space for the Bootstrap Allocator.
const BOOTSTRAP_HEAP_SIZE: usize = 2*1024*1024;

/// Alignment of pointers returned by the Bootstrap Allocator.
/// Note that you also have to align the HermitAllocatorInfo structure!
const BOOTSTRAP_HEAP_ALIGNMENT: usize = CACHE_LINE;

/// The Allocator structure is immutable, so we need this helper structure
/// for our allocator information.
#[repr(align(64))]
#[repr(C)]
struct AllocatorInfo {
	heap: [u8; BOOTSTRAP_HEAP_SIZE],
	index: usize
}

impl AllocatorInfo {
	const fn new() -> AllocatorInfo {
		AllocatorInfo {
			heap: [0xCC; BOOTSTRAP_HEAP_SIZE],
			index: 0
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
	let ptr = &mut ALLOCATOR_INFO.heap[ALLOCATOR_INFO.index] as *mut u8;
	debug!("Allocating {:#X} bytes at {:#X}, index {}", layout.size(), ptr as usize, ALLOCATOR_INFO.index);

	if ALLOCATOR_INFO.index + layout.size() >= BOOTSTRAP_HEAP_SIZE {
		panic!("Bootstrap Allocator Overflow! Increase BOOTSTRAP_HEAP_SIZE.");
	}

	// Bump the heap index and align it up to the next BOOTSTRAP_HEAP_ALIGNMENT boundary.
	ALLOCATOR_INFO.index = align_up!(ALLOCATOR_INFO.index + layout.size(), BOOTSTRAP_HEAP_ALIGNMENT);

	ptr
}

pub fn init() {
}
