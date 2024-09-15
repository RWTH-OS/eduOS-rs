extern "C" {
	pub fn main();
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
	crate::arch::x86::kernel::BOOT_INFO = Some(boot_info);

	main();

	loop {}
}
