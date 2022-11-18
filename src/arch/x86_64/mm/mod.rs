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
