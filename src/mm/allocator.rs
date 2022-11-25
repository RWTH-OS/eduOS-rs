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

use crate::arch::mm::VirtAddr;
use crate::arch::{BasePageSize, PageSize};
use crate::logging::*;
use crate::mm;
use crate::mm::freelist::{FreeList, FreeListEntry};
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;

/// Size of the preallocated space for the Bootstrap Allocator.
const BOOTSTRAP_HEAP_SIZE: usize = 2 * 1024 * 1024;
const HEAP_SIZE: usize = 8 * 1024 * 1024;

/// The Allocator structure is immutable, so we need this helper structure
/// for our allocator information.
#[repr(align(4096))]
#[repr(C)]
struct AllocatorInfo {
	heap: [u8; BOOTSTRAP_HEAP_SIZE],
	free_list: FreeList<VirtAddr>,
	index: usize,
	is_bootstrapping: bool,
}

impl AllocatorInfo {
	const fn new() -> AllocatorInfo {
		AllocatorInfo {
			heap: [0x00; BOOTSTRAP_HEAP_SIZE],
			free_list: FreeList::new(),
			index: 0,
			is_bootstrapping: true,
		}
	}

	fn switch_to_system_allocator(&mut self) {
		let size = align_up!(HEAP_SIZE, BasePageSize::SIZE);
		let addr = mm::allocate(size, true);
		let entry = FreeListEntry::new(addr, addr + size);
		self.free_list.list.push_back(entry);

		info!("Switching to the System Allocator");
		info!("Heap start at 0x{:x}", addr);

		self.is_bootstrapping = false;
	}
}

static mut ALLOCATOR_INFO: AllocatorInfo = AllocatorInfo::new();

pub struct Allocator;

unsafe impl<'a> GlobalAlloc for &'a Allocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		if ALLOCATOR_INFO.is_bootstrapping {
			let ptr = &mut ALLOCATOR_INFO.heap[align_up!(ALLOCATOR_INFO.index, layout.align())]
				as *mut u8;
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
		} else {
			ALLOCATOR_INFO
				.free_list
				.allocate(layout.size(), Some(layout.align()))
				.expect("Out of memory!")
				.as_mut_ptr()
		}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let address = VirtAddr::from_u64(ptr as u64);

		// We never deallocate memory of the Bootstrap Allocator.
		// It would only increase the management burden and we wouldn't save
		// any significant amounts of memory.
		// So check if this is a pointer allocated by the System Allocator.
		debug!("Deallocate {} bytes at {:#X}", layout.size(), ptr as usize);
		if !crate::arch::mm::is_kernel(address) {
			ALLOCATOR_INFO.free_list.deallocate(address, layout.size())
		}
	}
}

pub fn init() {
	unsafe {
		ALLOCATOR_INFO.switch_to_system_allocator();
	}
}
