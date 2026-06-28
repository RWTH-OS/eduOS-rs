//! Periodic timer based on the ARM generic timer (EL1 physical timer).
//!
//! The timer fires `TIMER_FREQ` times per second and drives the preemptive
//! scheduler. Its interrupt is delivered through the GIC as `gic::TIMER_INTID`.

use crate::consts::TIMER_FREQ;
use crate::logging::*;
use aarch64_cpu::registers::{Readable, Writeable, CNTFRQ_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};

/// Re-arm the down-counter for the next tick.
#[inline]
fn reload() {
	// CNTFRQ_EL0 holds the timer frequency in Hz.
	let interval = CNTFRQ_EL0.get() / u64::from(TIMER_FREQ);
	CNTP_TVAL_EL0.set(interval);
}

/// Start the periodic EL1 physical timer.
pub(crate) fn init() {
	debug!("initialize generic timer ({} Hz)", TIMER_FREQ);

	reload();
	// ENABLE = 1 (bit 0), IMASK = 0 (bit 1) => timer enabled, interrupt unmasked.
	CNTP_CTL_EL0.set(1);
}

/// Handle a timer interrupt by re-arming the timer for the next tick.
pub(crate) fn handle() {
	reload();
}
