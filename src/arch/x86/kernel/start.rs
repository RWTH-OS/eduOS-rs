use crate::arch::x86::kernel::processor::shutdown;

extern "C" {
	pub fn main() -> i32;
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
	crate::arch::x86::kernel::BOOT_INFO = Some(boot_info);

	let ret = main();

	shutdown(ret)
}
