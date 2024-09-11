// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::{init, processor};

#[cfg(all(target_arch = "x86", feature = "vga"))]
pub use self::x86::vga;

#[cfg(not(all(target_arch = "x86", feature = "vga")))]
pub use self::x86::serial;

// Implementations for x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;
