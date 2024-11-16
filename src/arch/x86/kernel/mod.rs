pub mod processor;
#[cfg(not(feature = "vga"))]
pub mod serial;
pub mod start;
#[cfg(feature = "vga")]
pub mod vga;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry.s"));

pub fn init() {
	processor::cpu_init();

	#[cfg(all(target_arch = "x86", feature = "vga"))]
	vga::init();
}
