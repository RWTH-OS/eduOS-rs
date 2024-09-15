mod gdt;
pub mod irq;
mod pit;
pub mod processor;
pub mod serial;
mod start;
pub mod switch;
pub mod task;

use bootloader::BootInfo;

pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();
}
