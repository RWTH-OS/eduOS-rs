use crate::arch::x86::kernel::BOOT_INFO;
use crate::scheduler::task::Stack;
use bootloader::bootinfo::MemoryRegionType;
use core::ops::Deref;

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
