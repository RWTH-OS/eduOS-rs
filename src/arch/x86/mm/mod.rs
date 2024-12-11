// Original version written by Colin Finck, RWTH Aachen University

pub mod paging;
pub mod physicalmem;
pub mod virtualmem;

use crate::arch::x86::kernel::BOOT_INFO;
use crate::consts::INTERRUPT_STACK_SIZE;
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

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) struct BootStack {
	start: VirtAddr,
	end: VirtAddr,
	ist_start: VirtAddr,
	ist_end: VirtAddr,
}

impl BootStack {
	pub const fn new(
		start: VirtAddr,
		end: VirtAddr,
		ist_start: VirtAddr,
		ist_end: VirtAddr,
	) -> Self {
		Self {
			start,
			end,
			ist_start,
			ist_end,
		}
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

	fn interrupt_top(&self) -> VirtAddr {
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "x86")] {
				self.ist_end - 16u32
			} else {
				self.ist_end - 16u64
			}
		}
	}

	fn interrupt_bottom(&self) -> VirtAddr {
		self.ist_start
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
					VirtAddr((BOOT_IST_STACK.0.as_ptr() as usize).try_into().unwrap()),
					VirtAddr(
						(BOOT_IST_STACK.0.as_ptr() as usize + INTERRUPT_STACK_SIZE)
							.try_into()
							.unwrap(),
					),
				);
			}
		}

		panic!("Unable to determine the kernel stack");
	}
}

#[allow(dead_code)]
pub(crate) fn is_kernel(addr: VirtAddr) -> bool {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::Kernel
				&& addr >= VirtAddr(i.range.start_frame_number * 0x1000)
				&& addr <= VirtAddr(i.range.end_frame_number * 0x1000)
			{
				return true;
			}

			if i.region_type == MemoryRegionType::KernelStack
				&& addr >= VirtAddr(i.range.start_frame_number * 0x1000)
				&& addr <= VirtAddr(i.range.end_frame_number * 0x1000)
			{
				return true;
			}
		}
	}

	false
}

pub(crate) fn get_memory_size() -> usize {
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

pub(crate) fn init() {
	paging::init();
	physicalmem::init();
	virtualmem::init();
}

#[repr(C, align(64))]
pub(crate) struct Aligned<T>(T);

impl<T> Aligned<T> {
	/// Constructor.
	pub const fn new(t: T) -> Self {
		Self(t)
	}
}

#[cfg(target_arch = "x86")]
pub(crate) const BOOT_STACK_SIZE: usize = 0x10000;
#[cfg(target_arch = "x86")]
#[link_section = ".data"]
pub(crate) static mut BOOT_STACK: Aligned<[u8; BOOT_STACK_SIZE]> =
	Aligned::new([0; BOOT_STACK_SIZE]);
pub(crate) static mut BOOT_IST_STACK: Aligned<[u8; INTERRUPT_STACK_SIZE]> =
	Aligned::new([0; INTERRUPT_STACK_SIZE]);

#[cfg(target_arch = "x86")]
pub(crate) fn get_boot_stack() -> BootStack {
	BootStack::new(
		unsafe { VirtAddr((BOOT_STACK.0.as_ptr() as usize).try_into().unwrap()) },
		unsafe {
			VirtAddr(
				(BOOT_STACK.0.as_ptr() as usize + BOOT_STACK_SIZE)
					.try_into()
					.unwrap(),
			)
		},
		unsafe { VirtAddr((BOOT_IST_STACK.0.as_ptr() as usize).try_into().unwrap()) },
		unsafe {
			VirtAddr(
				(BOOT_IST_STACK.0.as_ptr() as usize + INTERRUPT_STACK_SIZE)
					.try_into()
					.unwrap(),
			)
		},
	)
}
