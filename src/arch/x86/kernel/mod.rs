#[cfg(target_arch = "x86_64")]
pub(crate) mod processor;
#[cfg(not(feature = "vga"))]
pub(crate) mod serial;
mod start;
pub(crate) mod switch;
pub(crate) mod task;
#[cfg(feature = "vga")]
pub(crate) mod vga;

#[cfg(target_arch = "x86_64")]
use bootloader::BootInfo;
#[cfg(target_arch = "x86_64")]
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry32.s"));

pub(crate) fn init() {
	processor::cpu_init();

	#[cfg(all(target_arch = "x86", feature = "vga"))]
	vga::init();
}
