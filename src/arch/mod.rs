// Implementations for x86_64.
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
