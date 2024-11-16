/// Currently, eduOS supports only x86_64 (64 bit)
/// and x86 (32 bit) code. Both architecture are similar
/// and share the code in the directory x86

// Implementations for x86_64 and x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86_64::kernel::{irq, jump_to_user_land, processor, register_task, serial};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::{init, processor};

#[cfg(feature = "vga")]
pub(crate) use self::x86::kernel::vga;

#[cfg(not(feature = "vga"))]
pub(crate) use self::x86::kernel::serial;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::mm;
