//! Interrupt handling for aarch64.
//!
//! This installs the EL1 exception vector table (see `vectors.s`), provides the
//! `irq_*` primitives used by the rest of the kernel, and dispatches incoming
//! interrupts. The periodic timer interrupt drives the preemptive scheduler.

use crate::arch::aarch64::kernel::{gic, timer};
use crate::logging::*;
use crate::scheduler::{abort, get_current_taskid};
use crate::syscall::SYSHANDLER_TABLE;
use core::arch::{asm, global_asm};
use core::ptr::addr_of;

global_asm!(
	include_str!("vectors.s"),
	handle_exception = sym handle_exception,
);

extern "C" {
	/// Symbol of the exception vector table, defined in `vectors.s`.
	static vector_table_el1: u8;
}

/// Saved integer register state on exception entry. The layout has to match the
/// `SAVE_CONTEXT` macro in `vectors.s`.
#[repr(C)]
pub struct TrapFrame {
	/// general purpose registers x0..x30
	pub regs: [u64; 31],
	/// exception link register (return address)
	pub elr: u64,
	/// saved program status register
	pub spsr: u64,
}

/// Enable interrupts by clearing the IRQ mask bit (`I`) in `DAIF`.
pub fn irq_enable() {
	unsafe { asm!("msr daifclr, #2", options(nostack, nomem)) };
}

/// Disable interrupts by setting the IRQ mask bit (`I`) in `DAIF`.
pub fn irq_disable() {
	unsafe { asm!("msr daifset, #2", options(nostack, nomem)) };
}

/// Determines whether interrupts are currently enabled.
pub fn is_irq_enabled() -> bool {
	let daif: u64;
	unsafe { asm!("mrs {}, daif", out(reg) daif, options(nostack, nomem)) };
	// Bit 7 is the IRQ mask (`I`). A cleared bit means interrupts are enabled.
	daif & (1 << 7) == 0
}

/// Disable IRQs and return whether they were enabled before.
pub fn irq_nested_disable() -> bool {
	let was_enabled = is_irq_enabled();
	irq_disable();
	was_enabled
}

/// Re-enable IRQs only if they were enabled before the matching
/// [`irq_nested_disable`].
pub fn irq_nested_enable(was_enabled: bool) {
	if was_enabled {
		irq_enable();
	}
}

/// Install the exception vector table.
pub(crate) fn init() {
	debug!("install exception vector table");

	unsafe {
		asm!(
			"msr vbar_el1, {}",
			"isb",
			in(reg) addr_of!(vector_table_el1) as u64,
			options(nostack),
		);
	}
}

/// Common dispatcher called from every vector entry with the saved `frame` and
/// the `index` of the vector that was taken (0..15).
extern "C" fn handle_exception(frame: *mut TrapFrame, index: u64) {
	// Within a vector group the order is: 0 = Synchronous, 1 = IRQ, 2 = FIQ,
	// 3 = SError.
	if index & 0b11 == 1 {
		handle_irq();
		return;
	}

	let esr: u64;
	let far: u64;
	let elr: u64;
	unsafe {
		asm!("mrs {}, esr_el1", out(reg) esr, options(nostack, nomem));
		asm!("mrs {}, far_el1", out(reg) far, options(nostack, nomem));
		asm!("mrs {}, elr_el1", out(reg) elr, options(nostack, nomem));
	}

	// A synchronous exception from a lower EL (index 8) with exception class
	// 0b010101 is an `svc` from AArch64, i.e. a system call.
	if index == 8 && (esr >> 26) == 0b01_0101 {
		unsafe { handle_syscall(&mut *frame) };
		return;
	}

	error!(
		"Task {}: Unhandled exception (vector {}): ESR_EL1 {:#x}, FAR_EL1 {:#x}, ELR_EL1 {:#x}",
		get_current_taskid(),
		index,
		esr,
		far,
		elr
	);
	abort();
}

/// Dispatch a system call issued via `svc`. The system-call number is in `x8`,
/// the arguments in `x0`-`x5`, and the result is written back to `x0`.
unsafe fn handle_syscall(frame: &mut TrapFrame) {
	let no = frame.regs[8] as usize;

	if let Some(handler) = SYSHANDLER_TABLE.get(no) {
		// All handlers use the C calling convention; passing the maximum number
		// of argument registers is safe because the callee ignores unused ones.
		let handler: extern "C" fn(u64, u64, u64, u64, u64, u64) -> i64 =
			unsafe { core::mem::transmute(handler) };
		let ret = handler(
			frame.regs[0],
			frame.regs[1],
			frame.regs[2],
			frame.regs[3],
			frame.regs[4],
			frame.regs[5],
		);
		frame.regs[0] = ret as u64;
	}
}

/// Acknowledge the pending interrupt, dispatch it and signal completion to the
/// GIC. The timer interrupt triggers a reschedule.
fn handle_irq() {
	if let Some(intid) = gic::acknowledge() {
		if intid == gic::TIMER_INTID {
			// re-arm the timer and finish the interrupt before potentially
			// switching to another task
			timer::handle();
			gic::end(intid);
			crate::scheduler::schedule();
		} else {
			gic::end(intid);
		}
	}
}
