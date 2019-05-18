// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use arch::x86_64::mm::paging::{BasePageSize, PageSize};
use arch::x86_64::kernel::get_memory_size;
use collections::Node;
use mm;
use mm::freelist::{FreeList, FreeListEntry};
use mm::POOL;
use scheduler::DisabledPreemption;

static mut PHYSICAL_FREE_LIST: FreeList = FreeList::new();


fn detect_from_limits() -> Result<(), ()> {
	let limit = get_memory_size();

	if limit == 0 {
		return Err(());
	}

	let entry = Node::new(
		FreeListEntry {
			start: mm::kernel_end_address(),
			end: limit
		}
	);
	unsafe { PHYSICAL_FREE_LIST.list.push(entry); }

	Ok(())
}

pub fn init() {
	detect_from_limits().unwrap();
}

pub fn allocate(size: usize) -> usize {
	assert!(size > 0);
	assert!(size % BasePageSize::SIZE == 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size) };
	assert!(result.is_ok(), "Could not allocate {:#X} bytes of physical memory", size);
	result.unwrap()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> usize {
	assert!(size > 0);
	assert!(alignment > 0);
	assert!(size % alignment == 0, "Size {:#X} is not a multiple of the given alignment {:#X}", size, alignment);
	assert!(alignment % BasePageSize::SIZE == 0, "Alignment {:#X} is not a multiple of {:#X}", alignment, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe {
		POOL.maintain();
		PHYSICAL_FREE_LIST.allocate_aligned(size, alignment)
	};
	assert!(result.is_ok(), "Could not allocate {:#X} bytes of physical memory aligned to {} bytes", size, alignment);
	result.unwrap()
}

pub fn deallocate(physical_address: usize, size: usize) {
	assert!(physical_address >= mm::kernel_end_address(), "Physical address {:#X} is not >= KERNEL_END_ADDRESS", physical_address);
	assert!(size > 0);
	assert!(size % BasePageSize::SIZE == 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	unsafe {
		POOL.maintain();
		PHYSICAL_FREE_LIST.deallocate(physical_address, size);
	}
}
