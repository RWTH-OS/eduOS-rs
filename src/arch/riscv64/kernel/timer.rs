//! Periodic timer based on the RISC-V Sstc extension.
//!
//! The kernel runs in S-mode and programs its own timer through the
//! `stimecmp` CSR (Sstc). A supervisor timer interrupt is requested once
//! `time >= stimecmp`; we re-arm `stimecmp` on every tick to fire `TIMER_FREQ`
//! times per second and drive the preemptive scheduler. The M-mode shim in
//! `start.s` enables Sstc (`menvcfg.STCE`) and S-mode access to `time`
//! (`mcounteren.TM`).

use crate::consts::TIMER_FREQ;
use crate::logging::*;
use core::arch::asm;

/// Frequency of the `time` counter on the QEMU `virt` machine (10 MHz).
const TIMEBASE_FREQ: u64 = 10_000_000;
/// `sie.STIE` — supervisor timer interrupt enable.
const SIE_STIE: usize = 1 << 5;

/// Number of `time` ticks between two timer interrupts.
#[inline]
fn interval() -> u64 {
	TIMEBASE_FREQ / u64::from(TIMER_FREQ)
}

/// Re-arm `stimecmp` for the next tick. Writing a future value also clears the
/// pending supervisor timer interrupt.
#[inline]
fn reload() {
	let now: u64;
	unsafe {
		asm!("csrr {}, time", out(reg) now, options(nostack, nomem));
		asm!("csrw stimecmp, {}", in(reg) now + interval(), options(nostack, nomem));
	}
}

/// Start the periodic supervisor timer.
pub(crate) fn init() {
	debug!("initialize Sstc timer ({} Hz)", TIMER_FREQ);

	reload();
	// Enable the supervisor timer interrupt source (sie.STIE).
	unsafe { asm!("csrs sie, {}", in(reg) SIE_STIE, options(nostack, nomem)) };
}

/// Handle a timer interrupt by re-arming the timer for the next tick.
pub(crate) fn handle() {
	reload();
}
