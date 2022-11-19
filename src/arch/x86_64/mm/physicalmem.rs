// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::x86_64::kernel::BOOT_INFO;
use crate::arch::x86_64::mm::paging::{BasePageSize, PageSize};
use crate::logging::*;
use crate::mm::freelist::{FreeList, FreeListEntry};
use crate::scheduler::DisabledPreemption;
use core::convert::TryInto;
use core::ops::Deref;

static mut PHYSICAL_FREE_LIST: FreeList = FreeList::new();

pub fn init() {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == bootloader::bootinfo::MemoryRegionType::Usable {
				let entry = FreeListEntry {
					start: (i.range.start_frame_number * 0x1000).try_into().unwrap(),
					end: (i.range.end_frame_number * 0x1000).try_into().unwrap(),
				};

				debug!(
					"Add free physical regions 0x{:x} - 0x{:x}",
					entry.start, entry.end
				);
				PHYSICAL_FREE_LIST.list.push_back(entry);
			}
		}
	}
}

pub fn allocate(size: usize) -> usize {
	assert!(size > 0);
	assert!(
		size % BasePageSize::SIZE == 0,
		"Size {:#X} is not a multiple of {:#X}",
		size,
		BasePageSize::SIZE
	);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size, None) };
	assert!(
		result.is_ok(),
		"Could not allocate {:#X} bytes of physical memory",
		size
	);
	result.unwrap()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> usize {
	assert!(size > 0);
	assert!(alignment > 0);
	assert!(
		size % alignment == 0,
		"Size {:#X} is not a multiple of the given alignment {:#X}",
		size,
		alignment
	);
	assert!(
		alignment % BasePageSize::SIZE == 0,
		"Alignment {:#X} is not a multiple of {:#X}",
		alignment,
		BasePageSize::SIZE
	);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size, Some(alignment)) };
	assert!(
		result.is_ok(),
		"Could not allocate {:#X} bytes of physical memory aligned to {} bytes",
		size,
		alignment
	);
	result.unwrap()
}

pub fn deallocate(physical_address: usize, size: usize) {
	assert!(size > 0);
	assert!(
		size % BasePageSize::SIZE == 0,
		"Size {:#X} is not a multiple of {:#X}",
		size,
		BasePageSize::SIZE
	);

	let _preemption = DisabledPreemption::new();
	unsafe {
		PHYSICAL_FREE_LIST.deallocate(physical_address, size);
	}
}
