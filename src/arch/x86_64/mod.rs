pub mod serial;
pub mod processor;
pub mod task;
pub mod irq;
pub mod switch;
mod gdt;
mod pit;
mod start;

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();
}
