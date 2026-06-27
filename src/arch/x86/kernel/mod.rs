pub(crate) mod processor;
#[cfg(not(feature = "vga"))]
pub(crate) mod serial;
mod start;
pub(crate) mod switch;
pub(crate) mod task;
#[cfg(feature = "vga")]
pub(crate) mod vga;

use bootloader::BootInfo;
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub(crate) fn init() {
	processor::cpu_init();

	#[cfg(feature = "vga")]
	vga::init();
}
