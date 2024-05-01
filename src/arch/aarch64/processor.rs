#[no_mangle]
pub extern "C" fn shutdown(error_code: i32) -> ! {
	semihosting::process::exit(error_code)
}
