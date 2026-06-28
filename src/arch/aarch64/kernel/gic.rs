//! Minimal GICv3 bring-up using the `arm-gic` crate.
//!
//! The QEMU `virt` machine places the GICv3 distributor at `0x0800_0000` and the
//! redistributors at `0x080A_0000`. We only need the EL1 physical timer
//! interrupt, which is delivered as PPI 14 (INTID 30).

use crate::logging::*;
use arm_gic::gicv3::registers::{Gicd, GicrSgi};
use arm_gic::gicv3::{GicCpuInterface, GicV3};
use arm_gic::{IntId, InterruptGroup, UniqueMmioPointer};
use core::ptr::NonNull;

/// Base address of the GIC distributor on the QEMU `virt` machine.
const GICD_BASE: *mut Gicd = 0x0800_0000 as *mut Gicd;
/// Base address of the GIC redistributors on the QEMU `virt` machine.
const GICR_BASE: *mut GicrSgi = 0x080A_0000 as *mut GicrSgi;

/// INTID of the EL1 physical timer (PPI 14 => INTID 30).
pub const TIMER_INTID: IntId = IntId::ppi(14);

/// Initialize the GICv3 and enable the timer interrupt.
pub(crate) fn init() {
	debug!("initialize GICv3");

	// SAFETY: the base addresses are the fixed MMIO locations of the GICv3 on
	// the QEMU `virt` machine and are exclusively owned by the kernel.
	let gicd = unsafe { UniqueMmioPointer::new(NonNull::new(GICD_BASE).unwrap()) };
	let gicr = NonNull::new(GICR_BASE).unwrap();
	let mut gic = unsafe { GicV3::new(gicd, gicr, 1, false) };

	gic.setup(0);
	GicCpuInterface::set_priority_mask(0xff);
	gic.set_interrupt_priority(TIMER_INTID, Some(0), 0x00)
		.expect("failed to set timer interrupt priority");
	gic.enable_interrupt(TIMER_INTID, Some(0), true)
		.expect("failed to enable timer interrupt");
}

/// Acknowledge the highest priority pending group 1 interrupt, if any.
pub(crate) fn acknowledge() -> Option<IntId> {
	GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1)
}

/// Signal the end of handling for the given interrupt.
pub(crate) fn end(intid: IntId) {
	GicCpuInterface::end_interrupt(intid, InterruptGroup::Group1);
}
