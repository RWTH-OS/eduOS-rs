pub mod serial;
pub mod processor;
pub mod task;
pub mod irq;
mod gdt;
mod pit;
mod start;
mod switch;

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();
}
