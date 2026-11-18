//! Interrupt handling for RISC-V 64 (supervisor mode).
//!
//! This installs the supervisor trap vector (see `trap.s`), provides the `irq_*`
//! primitives used by the rest of the kernel, and dispatches incoming traps.
//! The periodic supervisor timer interrupt (Sstc) drives the preemptive
//! scheduler.

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

/// `sstatus.SIE` — global supervisor interrupt enable.
const SSTATUS_SIE: usize = 1 << 1;
/// `scause` interrupt code of the supervisor timer interrupt.
const IRQ_SUPERVISOR_TIMER: usize = 5;

/// Saved integer register state on trap entry. The layout has to match the
/// save/restore sequence in `trap.s`.
#[repr(C)]
pub struct TrapFrame {
	/// general purpose registers (ra, gp, tp, t0-t2, s0-s1, a0-a7, s2-s11, t3-t6)
	pub regs: [usize; 30],
	/// supervisor exception program counter
	pub sepc: usize,
	/// saved supervisor status register
	pub sstatus: usize,
}

/// Enable interrupts by setting `sstatus.SIE`.
pub fn irq_enable() {
	unsafe { asm!("csrs sstatus, {}", in(reg) SSTATUS_SIE, options(nostack, nomem)) };
}

/// Disable interrupts by clearing `sstatus.SIE`.
pub fn irq_disable() {
	unsafe { asm!("csrc sstatus, {}", in(reg) SSTATUS_SIE, options(nostack, nomem)) };
}

/// Determines whether interrupts are currently enabled.
pub fn is_irq_enabled() -> bool {
	let sstatus: usize;
	unsafe { asm!("csrr {}, sstatus", out(reg) sstatus, options(nostack, nomem)) };
	sstatus & SSTATUS_SIE != 0
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

/// Install the supervisor trap vector (direct mode).
pub(crate) fn init() {
	debug!("install trap vector");

	unsafe {
		asm!("csrw stvec, {}", in(reg) trap_entry as *const () as usize, options(nostack, nomem));
	}
}

/// Rust dispatcher called from `trap_entry` with a pointer to the saved frame.
extern "C" fn handle_trap(_frame: *mut TrapFrame) {
	let scause: usize;
	unsafe { asm!("csrr {}, scause", out(reg) scause, options(nostack, nomem)) };

	// The most significant bit distinguishes interrupts from exceptions.
	let is_interrupt = (scause >> (usize::BITS - 1)) != 0;
	let code = scause & !(1 << (usize::BITS - 1));

	if is_interrupt {
		if code == IRQ_SUPERVISOR_TIMER {
			// Re-arm the timer (this also clears the pending bit) before
			// potentially switching to another task.
			timer::handle();
			crate::scheduler::schedule();
		}
		return;
	}

	let sepc: usize;
	let stval: usize;
	unsafe {
		asm!("csrr {}, sepc", out(reg) sepc, options(nostack, nomem));
		asm!("csrr {}, stval", out(reg) stval, options(nostack, nomem));
	}

	panic!(
		"Unhandled exception: scause {:#x}, sepc {:#x}, stval {:#x}",
		scause, sepc, stval
	);
}
