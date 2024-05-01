extern "C" {
	fn main();
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
	main();

	loop {}
}
