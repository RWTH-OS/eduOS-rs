// Implementations for x86_64.
#[cfg(target_arch = "x86_64")]
pub mod x86;

// Implementations for aarch64.
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// Implementations for riscv64.
#[cfg(target_arch = "riscv64")]
pub mod riscv64;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::kernel::{init, processor};

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::processor;

#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::kernel::processor;

#[cfg(all(target_arch = "x86_64", feature = "vga"))]
pub(crate) use self::x86::kernel::vga;

#[cfg(all(target_arch = "x86_64", not(feature = "vga")))]
pub(crate) use self::x86::kernel::serial;

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::serial;

#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::kernel::serial;
