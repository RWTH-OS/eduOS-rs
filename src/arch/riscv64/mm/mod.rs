use crate::scheduler::task::Stack;
pub use memory_addresses::{PhysAddr, VirtAddr};

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

#[repr(C, align(64))]
pub(crate) struct Aligned<T>(T);

impl<T> Aligned<T> {
	/// Constructor.
	pub const fn new(t: T) -> Self {
		Self(t)
	}
}

pub(crate) const BOOT_STACK_SIZE: usize = 0x16000;
#[link_section = ".data"]
pub(crate) static mut BOOT_STACK: Aligned<[u8; BOOT_STACK_SIZE]> =
	Aligned::new([0; BOOT_STACK_SIZE]);

pub(crate) fn get_boot_stack() -> BootStack {
	BootStack::new(
		unsafe { VirtAddr::new_unsafe((BOOT_STACK.0.as_ptr() as usize).try_into().unwrap()) },
		unsafe {
			VirtAddr::new_unsafe(
				(BOOT_STACK.0.as_ptr() as usize + BOOT_STACK_SIZE)
					.try_into()
					.unwrap(),
			)
		},
	)
}
