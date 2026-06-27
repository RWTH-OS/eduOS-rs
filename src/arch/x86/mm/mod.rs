use crate::scheduler::task::Stack;

pub use x86::bits64::paging::VAddr as VirtAddr;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) struct BootStack {
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

pub(crate) fn get_boot_stack() -> BootStack {
	use crate::arch::x86::kernel::BOOT_INFO;
	use bootloader::bootinfo::MemoryRegionType;
	use core::ops::Deref;

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
