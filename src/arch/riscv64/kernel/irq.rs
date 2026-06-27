//! Interrupt handling for RISC-V 64 (machine mode).
//!
//! This installs the machine trap vector (see `trap.s`), provides the `irq_*`
//! primitives used by the rest of the kernel, and dispatches incoming traps.
//! The periodic CLINT timer interrupt drives the preemptive scheduler.

use crate::arch::riscv64::kernel::timer;
use crate::logging::*;
use core::arch::{asm, global_asm};

global_asm!(
	include_str!("trap.s"),
	handle_trap = sym handle_trap,
);

extern "C" {
	/// Symbol of the trap vector, defined in `trap.s`.
	fn trap_entry();
}

/// `mstatus.MIE` — global machine interrupt enable.
const MSTATUS_MIE: usize = 1 << 3;
/// `mcause` interrupt code of the machine timer interrupt.
const IRQ_MACHINE_TIMER: usize = 7;

/// Saved integer register state on trap entry. The layout has to match the
/// save/restore sequence in `trap.s`.
#[repr(C)]
pub struct TrapFrame {
	/// general purpose registers (ra, gp, tp, t0-t2, s0-s1, a0-a7, s2-s11, t3-t6)
	pub regs: [usize; 30],
	/// machine exception program counter
	pub mepc: usize,
	/// saved machine status register
	pub mstatus: usize,
}

/// Enable interrupts by setting `mstatus.MIE`.
pub fn irq_enable() {
	unsafe { asm!("csrs mstatus, {}", in(reg) MSTATUS_MIE, options(nostack, nomem)) };
}

/// Disable interrupts by clearing `mstatus.MIE`.
pub fn irq_disable() {
	unsafe { asm!("csrc mstatus, {}", in(reg) MSTATUS_MIE, options(nostack, nomem)) };
}

/// Determines whether interrupts are currently enabled.
pub fn is_irq_enabled() -> bool {
	let mstatus: usize;
	unsafe { asm!("csrr {}, mstatus", out(reg) mstatus, options(nostack, nomem)) };
	mstatus & MSTATUS_MIE != 0
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

/// Install the machine trap vector (direct mode).
pub(crate) fn init() {
	debug!("install trap vector");

	unsafe {
		asm!("csrw mtvec, {}", in(reg) trap_entry as *const () as usize, options(nostack, nomem));
	}
}

/// Rust dispatcher called from `trap_entry` with a pointer to the saved frame.
extern "C" fn handle_trap(_frame: *mut TrapFrame) {
	let mcause: usize;
	unsafe { asm!("csrr {}, mcause", out(reg) mcause, options(nostack, nomem)) };

	// The most significant bit distinguishes interrupts from exceptions.
	let is_interrupt = (mcause >> (usize::BITS - 1)) != 0;
	let code = mcause & !(1 << (usize::BITS - 1));

	if is_interrupt {
		if code == IRQ_MACHINE_TIMER {
			// Re-arm the timer (this also clears the pending bit) before
			// potentially switching to another task.
			timer::handle();
			crate::scheduler::schedule();
		}
		return;
	}

	let mepc: usize;
	let mtval: usize;
	unsafe {
		asm!("csrr {}, mepc", out(reg) mepc, options(nostack, nomem));
		asm!("csrr {}, mtval", out(reg) mtval, options(nostack, nomem));
	}

	panic!(
		"Unhandled exception: mcause {:#x}, mepc {:#x}, mtval {:#x}",
		mcause, mepc, mtval
	);
}
