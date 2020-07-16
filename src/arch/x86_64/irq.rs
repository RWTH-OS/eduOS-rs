// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// Enable Interrupts
pub fn irq_enable() {
	unsafe { llvm_asm!("sti" ::: "memory" : "volatile") };
}

/// Disable Interrupts
pub fn irq_disable() {
	unsafe { llvm_asm!("cli" ::: "memory" : "volatile") };
}

/// Determines, if the interrupt flags (IF) is set
pub fn is_irq_enabled() -> bool {
	let rflags: u64;

	unsafe { llvm_asm!("pushf; pop $0": "=r"(rflags) :: "memory" : "volatile") };
	if (rflags & (1u64 << 9)) != 0 {
		return true;
	}

	false
}

/// Disable IRQs (nested)
///
/// Disable IRQs when unsure if IRQs were enabled at all.
/// This function together with irq_nested_enable can be used
/// in situations when interrupts shouldn't be activated if they
/// were not activated before calling this function.
pub fn irq_nested_disable() -> bool {
	let was_enabled = is_irq_enabled();
	irq_disable();
	was_enabled
}

/// Enable IRQs (nested)
///
/// Can be used in conjunction with irq_nested_disable() to only enable
/// interrupts again if they were enabled before.
pub fn irq_nested_enable(was_enabled: bool) {
	if was_enabled == true {
		irq_enable();
	}
}
