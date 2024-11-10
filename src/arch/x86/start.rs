use crate::arch::x86::processor::shutdown;

extern "C" {
	fn main() -> i32;
}

#[cfg(not(test))]
#[no_mangle]
/// # Safety
///
/// This function is the entry point of the kernel.
/// The kernel itself should not call this function.
pub unsafe extern "C" fn _start() -> ! {
	let ret = main();
	// shutdown system
	shutdown(ret);
}
