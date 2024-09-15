use core::arch::asm;

/// Enable Interrupts
pub fn irq_enable() {
	unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
}

/// Disable Interrupts
pub fn irq_disable() {
	unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
}

/// Determines, if the interrupt flags (IF) is set
pub fn is_irq_enabled() -> bool {
	let rflags: u64;

	unsafe { asm!("pushf; pop {}", lateout(reg) rflags, options(nomem, nostack, preserves_flags)) };
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
