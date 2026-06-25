use semihosting::process::exit;

/// Shutdown the system
#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
	exit(error_code)
}
