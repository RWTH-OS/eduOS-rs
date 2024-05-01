use core::arch::asm;

extern "C" {
	fn main() -> !;
}

#[cfg(not(test))]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() -> ! {
	// init 1G stack stack in RAM area (>0x40000000 for qemu virt device)
	asm!(
		"movz x1, 0x5000, lsl 16",
		"mov sp, x1",
		// Jump to Rust code
		"b {main}",
		main = sym main,
		options(noreturn),
	);
}
