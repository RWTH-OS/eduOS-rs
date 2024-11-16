// Implementations for x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::{init, processor};

#[cfg(feature = "vga")]
pub(crate) use self::x86::kernel::vga;

#[cfg(not(feature = "vga"))]
pub(crate) use self::x86::kernel::serial;
