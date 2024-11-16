#[cfg(not(test))]
use alloc::alloc::Layout;

#[cfg(not(test))]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
	panic!(
		"[!!!OOM!!!] Memory allocation of {} bytes failed",
		layout.size()
	);
}
