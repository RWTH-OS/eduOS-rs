pub mod processor;
pub mod serial;
mod start;
pub mod switch;
pub mod task;

use bootloader::BootInfo;

pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;
