pub mod irq;
pub(crate) mod processor;
pub(crate) mod serial;
pub(crate) mod start;
pub(crate) mod switch;
pub(crate) mod task;
pub mod timer;

/// Initialize the architecture specific interrupt infrastructure.
///
/// This installs the trap vector and starts the periodic CLINT timer that
/// drives the preemptive scheduler. Interrupts stay globally masked
/// (`mstatus.MIE` is cleared at reset) until the kernel explicitly enables them
/// via `irq::irq_enable`.
pub(crate) fn init() {
	irq::init();
	timer::init();
}
