pub mod freelist;

use crate::arch;
use crate::arch::mm::get_memory_size;
use crate::logging::*;
#[cfg(not(test))]
use alloc::alloc::Layout;

pub fn init() {
	info!("Memory size {} MByte", get_memory_size() >> 20);

	arch::mm::init();
}

#[cfg(not(test))]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
	println!(
		"[!!!OOM!!!] Memory allocation of {} bytes failed",
		layout.size()
	);

	loop {}
}
