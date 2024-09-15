pub mod processor;
pub mod serial;
#[cfg(target_arch = "x86_64")]
mod start;
pub(crate) mod switch;
pub(crate) mod task;

#[cfg(target_arch = "x86_64")]
use bootloader::BootInfo;
#[cfg(target_arch = "x86_64")]
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry32.s"));
