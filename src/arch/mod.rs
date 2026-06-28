// eduOS supports x86_64, aarch64 and riscv64.

// Implementations for x86_64 and x86.
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
pub use self::x86::kernel::{irq, jump_to_user_land};

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::kernel::{irq, jump_to_user_land};

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::kernel::{irq, jump_to_user_land};

#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::kernel::{init, processor, register_task};

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::{init, processor, register_task};

#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::kernel::{init, processor, register_task};

#[cfg(all(target_arch = "x86_64", feature = "vga"))]
pub(crate) use self::x86::kernel::vga;

#[cfg(all(target_arch = "x86_64", not(feature = "vga")))]
pub(crate) use self::x86::kernel::serial;

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::serial;

#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::kernel::serial;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::kernel::switch::switch;

#[cfg(target_arch = "aarch64")]
pub(crate) use self::aarch64::kernel::switch::switch;

#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::mm;

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::mm;

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::mm;
