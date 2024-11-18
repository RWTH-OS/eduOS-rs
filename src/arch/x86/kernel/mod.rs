pub(crate) mod processor;
#[cfg(not(feature = "vga"))]
pub(crate) mod serial;
pub(crate) mod start;
#[cfg(feature = "vga")]
pub(crate) mod vga;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry.s"));

pub(crate) fn init() {
	processor::cpu_init();

	#[cfg(feature = "vga")]
	vga::init();
}
