mod gdt;
pub mod irq;
mod pit;
pub mod processor;
#[cfg(not(all(target_arch = "x86", feature = "vga")))]
pub mod serial;
#[cfg(target_arch = "x86_64")]
mod start;
pub(crate) mod switch;
pub(crate) mod task;
#[cfg(all(target_arch = "x86", feature = "vga"))]
pub mod vga;

#[cfg(target_arch = "x86_64")]
use bootloader::BootInfo;
#[cfg(target_arch = "x86_64")]
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry32.s"));

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();

	#[cfg(all(target_arch = "x86", feature = "vga"))]
	vga::init();
}
