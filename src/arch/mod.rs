#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::kernel::{init, irq, jump_to_user_land, processor, register_task, serial};

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use self::x86::mm;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::mm::paging::{
	drop_user_space, get_kernel_root_page_table, BasePageSize, PageSize,
};
