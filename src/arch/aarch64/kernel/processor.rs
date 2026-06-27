use core::arch::asm;

/// PSCI `SYSTEM_OFF` function ID (SMC Calling Convention).
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;

/// The halt function stops the processor until the next interrupt arrives
pub(crate) fn halt() {
	unsafe {
		asm!("wfi", options(nostack, nomem),);
	}
}

/// Shutdown the system
///
/// QEMU's `virt` machine provides PSCI through the hypervisor call (`hvc`)
/// conduit, so `SYSTEM_OFF` powers the machine off and terminates QEMU.
#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
	unsafe {
		asm!("hvc #0", in("x0") PSCI_SYSTEM_OFF, options(nostack, nomem));
	}

	// The call above terminates QEMU; loop just in case it does not.
	loop {
		halt();
	}
}
