use crate::arch::x86::processor::shutdown;

extern "C" {
	fn main() -> i32;
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
	let ret = main();
	// shutdown system
	shutdown(ret);
}
