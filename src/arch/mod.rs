// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::{processor, serial};

// Implementations for x86_64.
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// Export our platform-specific modules.
#[cfg(target_arch = "x86")]
pub use self::x86_64::{processor, serial};

// Implementations for x86_64.
#[cfg(target_arch = "x86")]
pub mod x86_64;
