// eduOS supports x86_64.

// Implementations for x86_64.
#[cfg(target_arch = "x86_64")]
pub mod x86;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::kernel::{init, processor, register_task, switch::switch};

#[cfg(target_arch = "x86_64")]
pub use self::x86::kernel::{irq, jump_to_user_land};

#[cfg(feature = "vga")]
pub(crate) use self::x86::kernel::vga;

#[cfg(not(feature = "vga"))]
pub(crate) use self::x86::kernel::serial;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::mm;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub(crate) use self::x86::mm::paging::{
	drop_user_space, get_kernel_root_page_table, BasePageSize, PageSize,
};
