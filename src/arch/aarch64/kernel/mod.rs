pub mod gic;
pub mod irq;
pub(crate) mod processor;
pub(crate) mod serial;
pub(crate) mod start;
pub(crate) mod switch;
pub(crate) mod task;
pub mod timer;

/// Initialize the architecture specific interrupt infrastructure.
///
/// This installs the exception vector table, brings up the GICv3 interrupt
/// controller and starts the periodic timer that drives the preemptive
/// scheduler. Interrupts stay masked (see `start::pre_init`) until the kernel
/// explicitly enables them via `irq::irq_enable`.
pub(crate) fn init() {
	irq::init();
	gic::init();
	timer::init();
}
