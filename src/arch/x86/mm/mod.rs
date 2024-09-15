// Original version written by Colin Finck, RWTH Aachen University

pub mod paging;
pub mod physicalmem;
pub mod virtualmem;

use crate::arch::x86::kernel::BOOT_INFO;
use crate::scheduler::task::Stack;
use bootloader::bootinfo::MemoryRegionType;
use core::convert::TryInto;
use core::ops::Deref;
#[cfg(target_arch = "x86")]
pub use x86::bits32::paging::VAddr as VirtAddr;
#[cfg(target_arch = "x86_64")]
pub use x86::bits64::paging::PAddr as PhysAddr;
#[cfg(target_arch = "x86_64")]
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
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "x86")] {
				self.end - 16u32
			} else {
				self.end - 16u64
			}
		}
	}

	fn bottom(&self) -> VirtAddr {
		self.start
	}
}

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
extern "C" {
	static BOOT_STACK: usize;
}

#[cfg(target_arch = "x86")]
pub(crate) fn get_boot_stack() -> BootStack {
	BootStack::new(
		unsafe { VirtAddr(BOOT_STACK.try_into().unwrap()) },
		unsafe { VirtAddr((BOOT_STACK + 0x1000).try_into().unwrap()) },
	)
}
