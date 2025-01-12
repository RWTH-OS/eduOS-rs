/// Currently, eduOS supports only x86_64 (64 bit)
/// and x86 (32 bit) code. Both architecture are similar
/// and share the code in the directory x86

// Implementations for x86_64 and x86.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::kernel::{init, processor, register_task, switch::switch};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::kernel::irq;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::load_application;

#[cfg(feature = "vga")]
pub(crate) use self::x86::kernel::vga;

#[cfg(not(feature = "vga"))]
pub(crate) use self::x86::kernel::serial;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::mm;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use self::x86::mm::paging::{
	drop_user_space, get_kernel_root_page_table, BasePageSize, PageSize,
};
