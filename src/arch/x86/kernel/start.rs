use crate::arch;
use crate::arch::x86::kernel::processor::shutdown;

extern "C" {
	pub fn main() -> i32;
}

#[cfg(not(test))]
#[no_mangle]
/// # Safety
///
/// This function is the entry point of the kernel.
/// The kernel itself should not call this function.
pub unsafe extern "C" fn _start() -> ! {
	arch::init();

	let ret = main();

	shutdown(ret)
}

