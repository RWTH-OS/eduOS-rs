/// Currently, eduOS supports only x86_64 (64 bit)
/// and x86 (32 bit) code. Both architecture are similar
/// and share the code in the directory x86

// Implementations for x86_64 and x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::kernel::{irq, processor, serial};

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::mm;

pub fn init() {
	processor::cpu_init();
}
