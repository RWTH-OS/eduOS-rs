// Copyright (c) 2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod allocator;
pub mod freelist;
mod nodepool;

use alloc::alloc::Layout;
use arch::mm::paging::{BasePageSize, PageSize, PageTableEntryFlags};
use mm::nodepool::NodePool;
use scheduler::DisabledPreemption;
use logging::*;
use arch;

extern "C" {
	static kernel_start: u8;
	static kernel_end: u8;
}

pub static mut POOL: NodePool = NodePool::new();

/// Physical and virtual address of the first 2 MiB page that maps the kernel.
/// Can be easily accessed through kernel_start_address()
static mut KERNEL_START_ADDRESS: usize = 0;

/// Physical and virtual address of the first page after the kernel.
/// Can be easily accessed through kernel_end_address()
static mut KERNEL_END_ADDRESS: usize = 0;

pub fn kernel_start_address() -> usize {
	unsafe { KERNEL_START_ADDRESS }
}

pub fn kernel_end_address() -> usize {
	unsafe { KERNEL_END_ADDRESS }
}

pub fn allocate(size: usize, execute_disable: bool) -> usize {
	let _preemption = DisabledPreemption::new();

	let physical_address = arch::mm::physicalmem::allocate(size);
	let virtual_address = arch::mm::virtualmem::allocate(size);

	let count = size / BasePageSize::SIZE;
	let mut flags = PageTableEntryFlags::empty();
	flags.normal().writable();
	if execute_disable {
		flags.execute_disable();
	}
	arch::mm::paging::map::<BasePageSize>(virtual_address, physical_address, count, flags);

	virtual_address
}

pub fn deallocate(virtual_address: usize, size: usize) {
	let _preemption = DisabledPreemption::new();

	if let Some(entry) = arch::mm::paging::get_page_table_entry::<BasePageSize>(virtual_address) {
		arch::mm::virtualmem::deallocate(virtual_address, size);
		arch::mm::physicalmem::deallocate(entry.address(), size);
	} else {
		panic!("No page table entry for virtual address {:#X}", virtual_address);
	}
}

pub fn init() {
	let image_size;

	// Calculate the start and end addresses of the 2 MiB page(s) that map the kernel.
	unsafe {
		image_size = &kernel_end as *const u8 as usize - &kernel_start as *const u8 as usize;
		KERNEL_START_ADDRESS = align_down!(&kernel_start as *const u8 as usize, arch::mm::paging::LargePageSize::SIZE);
		KERNEL_END_ADDRESS = align_up!(&kernel_end as *const u8 as usize, arch::mm::paging::LargePageSize::SIZE);
	}

	info!("Memory size {} MByte", arch::get_memory_size() >> 20);
	info!("Kernel start address 0x{:x}", kernel_start_address());
	info!("Kernel end address 0x{:x}", kernel_end_address());
	info!("Kernel image size {} KByte", image_size >> 10);

	arch::mm::init();
	self::allocator::init();
}

#[cfg(not(test))]
#[lang = "oom"]
#[no_mangle]
pub fn rust_oom(layout: Layout) -> ! {
        println!("[!!!OOM!!!] Memory allocation of {} bytes failed", layout.size());

		loop {}
}
