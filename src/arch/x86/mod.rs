pub mod processor;
#[cfg(not(all(target_arch = "x86", feature = "vga")))]
pub mod serial;
#[cfg(target_arch = "x86_64")]
pub mod start;
#[cfg(all(target_arch = "x86", feature = "vga"))]
pub mod vga;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry.s"));

pub fn init() {
	processor::cpu_init();

	#[cfg(all(target_arch = "x86", feature = "vga"))]
	vga::init();
}
