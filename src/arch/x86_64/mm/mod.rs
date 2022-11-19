// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod paging;
pub mod physicalmem;
pub mod virtualmem;

use crate::arch::x86_64::kernel::BOOT_INFO;
use crate::scheduler::task::Stack;
use bootloader::bootinfo::MemoryRegionType;
use core::convert::TryInto;
use core::ops::Deref;

pub use x86::bits64::paging::PAddr as PhysAddr;
pub use x86::bits64::paging::VAddr as VirtAddr;

#[derive(Copy, Clone)]
pub struct BootStack {
	start: VirtAddr,
	end: VirtAddr,
}

impl BootStack {
	pub const fn new(start: VirtAddr, end: VirtAddr) -> Self {
		Self { start, end }
	}
}

impl Stack for BootStack {
	fn top(&self) -> VirtAddr {
		self.end - 16u64
	}

	fn bottom(&self) -> VirtAddr {
		self.start
	}
}

pub fn get_boot_stack() -> BootStack {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::KernelStack {
				return BootStack::new(
					VirtAddr(i.range.start_frame_number * 0x1000),
					VirtAddr(i.range.end_frame_number * 0x1000),
				);
			}
		}

		panic!("Unable to determine the kernel stack");
	}
}

pub fn is_kernel(addr: VirtAddr) -> bool {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::Kernel {
				if addr >= VirtAddr(i.range.start_frame_number * 0x1000)
					&& addr <= VirtAddr(i.range.end_frame_number * 0x1000)
				{
					return true;
				}
			}

			if i.region_type == MemoryRegionType::KernelStack {
				if addr >= VirtAddr(i.range.start_frame_number * 0x1000)
					&& addr <= VirtAddr(i.range.end_frame_number * 0x1000)
				{
					return true;
				}
			}
		}
	}

	false
}

pub fn get_memory_size() -> usize {
	let mut sz: u64 = 0;

	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			match i.region_type {
				MemoryRegionType::Usable
				| MemoryRegionType::InUse
				| MemoryRegionType::Kernel
				| MemoryRegionType::KernelStack
				| MemoryRegionType::PageTable
				| MemoryRegionType::Bootloader
				| MemoryRegionType::FrameZero
				| MemoryRegionType::BootInfo
				| MemoryRegionType::Package => {
					sz += (i.range.end_frame_number - i.range.start_frame_number) * 0x1000
				}
				_ => {}
			}
		}
	}

	sz.try_into().unwrap()
}

pub fn init() {
	paging::init();
	physicalmem::init();
	virtualmem::init();
}
