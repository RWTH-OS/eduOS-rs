//! Periodic timer based on the RISC-V machine timer (CLINT).
//!
//! On QEMU's `virt` machine the CLINT lives at `0x0200_0000` and its `mtime`
//! register increments at 10 MHz. A machine timer interrupt is requested once
//! `mtime >= mtimecmp`; we re-arm `mtimecmp` on every tick to fire `TIMER_FREQ`
//! times per second and drive the preemptive scheduler.

use crate::consts::TIMER_FREQ;
use crate::logging::*;
use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

/// Base address of the CLINT on the QEMU `virt` machine.
const CLINT_BASE: usize = 0x0200_0000;
/// `mtimecmp` register of hart 0.
const MTIMECMP: *mut u64 = (CLINT_BASE + 0x4000) as *mut u64;
/// `mtime` register.
const MTIME: *const u64 = (CLINT_BASE + 0xBFF8) as *const u64;
/// Frequency of the `mtime` counter on the QEMU `virt` machine (10 MHz).
const TIMEBASE_FREQ: u64 = 10_000_000;
/// `mie.MTIE` — machine timer interrupt enable.
const MIE_MTIE: usize = 1 << 7;

/// Number of `mtime` ticks between two timer interrupts.
#[inline]
fn interval() -> u64 {
	TIMEBASE_FREQ / u64::from(TIMER_FREQ)
}

/// Re-arm `mtimecmp` for the next tick. Writing a future value also clears the
/// pending machine timer interrupt.
#[inline]
fn reload() {
	let now = unsafe { read_volatile(MTIME) };
	unsafe { write_volatile(MTIMECMP, now + interval()) };
}

/// Start the periodic machine timer.
pub(crate) fn init() {
	debug!("initialize CLINT timer ({} Hz)", TIMER_FREQ);

	reload();
	// Enable the machine timer interrupt source (mie.MTIE).
	unsafe { asm!("csrs mie, {}", in(reg) MIE_MTIE, options(nostack, nomem)) };
}

/// Handle a timer interrupt by re-arming the timer for the next tick.
pub(crate) fn handle() {
	reload();
}
