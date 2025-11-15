/// Currently, eduOS supports only x86_64 (64 bit)
/// and x86 (32 bit) code. Both architecture are similar
/// and share the code in the directory x86

// Implementations for x86_64 and x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Implementations for aarch64.
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::{init, processor};

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::processor;

#[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), feature = "vga"))]
pub(crate) use self::x86::kernel::vga;

#[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), not(feature = "vga")))]
pub(crate) use self::x86::kernel::serial;

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::serial;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::mm;

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::mm;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::switch::switch;

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::mm;