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

#[derive(Copy, Clone)]
pub struct BootStack {
	start: usize,
	end: usize,
}

impl BootStack {
	pub const fn new(start: usize, end: usize) -> Self {
		Self { start, end }
	}
}

impl Stack for BootStack {
	fn top(&self) -> usize {
		self.end - 16
	}

	fn bottom(&self) -> usize {
		self.start
	}
}

pub fn get_boot_stack() -> BootStack {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::KernelStack {
				return BootStack::new(
					(i.range.start_frame_number * 0x1000).try_into().unwrap(),
					(i.range.end_frame_number * 0x1000).try_into().unwrap(),
				);
			}
		}

		panic!("Unable to determine the kernel stack");
	}
}

pub fn is_kernel(addr: usize) -> bool {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::Kernel {
				if addr >= (i.range.start_frame_number * 0x1000).try_into().unwrap()
					&& addr <= (i.range.end_frame_number * 0x1000).try_into().unwrap()
				{
					return true;
				}
			}

			if i.region_type == MemoryRegionType::KernelStack {
				if addr >= (i.range.start_frame_number * 0x1000).try_into().unwrap()
					&& addr <= (i.range.end_frame_number * 0x1000).try_into().unwrap()
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
