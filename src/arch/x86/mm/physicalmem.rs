use crate::arch::x86::kernel::BOOT_INFO;
use crate::arch::x86::mm::paging::{BasePageSize, PageSize};
use crate::arch::x86::mm::PhysAddr;
use crate::logging::*;
use crate::mm::freelist::{FreeList, FreeListEntry};
use crate::scheduler::DisabledPreemption;
use core::ops::Deref;

static mut PHYSICAL_FREE_LIST: FreeList<PhysAddr> = FreeList::new();

pub(crate) fn init() {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == bootloader::bootinfo::MemoryRegionType::Usable {
				let entry = FreeListEntry {
					start: (i.range.start_frame_number * 0x1000).into(),
					end: (i.range.end_frame_number * 0x1000).into(),
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

pub fn allocate(size: usize) -> PhysAddr {
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

pub fn allocate_aligned(size: usize, alignment: usize) -> PhysAddr {
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

pub fn deallocate(physical_address: PhysAddr, size: usize) {
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
