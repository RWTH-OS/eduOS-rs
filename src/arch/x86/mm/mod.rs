use crate::scheduler::task::Stack;

#[cfg(target_arch = "x86")]
pub use x86::bits32::paging::VAddr as VirtAddr;
#[cfg(target_arch = "x86_64")]
pub use x86::bits64::paging::VAddr as VirtAddr;

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

#[cfg(target_arch = "x86")]
extern "C" {
	static BOOT_STACK: usize;
}

#[cfg(target_arch = "x86")]
pub(crate) fn get_boot_stack() -> BootStack {
	BootStack::new(
		unsafe { VirtAddr(BOOT_STACK.try_into().unwrap()) },
		unsafe { VirtAddr((BOOT_STACK + 0x3000).try_into().unwrap()) },
	)
}
