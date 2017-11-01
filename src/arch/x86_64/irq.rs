// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use logging::*;
use scheduler::*;
use arch::x86_64::task::State;
use x86::bits64::irq::IdtEntry;
use x86::shared::dtables::{DescriptorTablePointer,lidt};
use x86::shared::PrivilegeLevel;
use x86::shared::paging::VAddr;
use cpuio::outb;

/// Maximum possible number of interrupts
const IDT_ENTRY_COUNT: usize = 256;
const KERNEL_CODE_SELECTOR: u16 = 0x08;

#[allow(dead_code)]
extern {
	/// Interrupt handlers => see irq_handler.asm
	static interrupt_handlers: [*const u8; IDT_ENTRY_COUNT];
}

/// An Interrupt Descriptor Table which specifies how to respond to each
/// interrupt.
#[repr(C, packed)]
struct Idt {
	pub table: [IdtEntry; IDT_ENTRY_COUNT]
}

impl Idt {
	pub const fn new() -> Idt {
		Idt {
			table: [IdtEntry::MISSING; IDT_ENTRY_COUNT]
		}
	}
}

/// our global Interrupt Descriptor Table .
static mut IDT: Idt = Idt::new();

/// Normally, IRQs 0 to 7 are mapped to entries 8 to 15. This
/// is a problem in protected mode, because IDT entry 8 is a
/// Double Fault! Without remapping, every time IRQ0 fires,
/// you get a Double Fault Exception, which is NOT what's
/// actually happening. We send commands to the Programmable
/// Interrupt Controller (PICs - also called the 8259's) in
/// order to make IRQ0 to 15 be remapped to IDT entries 32 to
/// 47
unsafe fn irq_remap()
{
	outb(0x11, 0x20);
	outb(0x11, 0xA0);
	outb(0x20, 0x21);
	outb(0x28, 0xA1);
	outb(0x04, 0x21);
	outb(0x02, 0xA1);
	outb(0x01, 0x21);
	outb(0x01, 0xA1);
	outb(0x0, 0x21);
	outb(0x0, 0xA1);
}

pub fn init() {
	info!("initialize interrupt descriptor table");

	unsafe {
		irq_remap();

		for i in 0..IDT_ENTRY_COUNT {
				IDT.table[i] = IdtEntry::new(VAddr::from_usize(interrupt_handlers[i] as usize),
			KERNEL_CODE_SELECTOR, PrivilegeLevel::Ring0, true);
		}

		// load address of the IDT
		let idtr: DescriptorTablePointer<IdtEntry> = DescriptorTablePointer::new_idtp(&IDT.table);
		lidt(&idtr)
	}
}

/// Enable Interrupts
#[inline(always)]
pub fn irq_enable() {
	unsafe { asm!("sti" ::: "memory" : "volatile") };
}

/// Disable Interrupts
#[inline(always)]
pub fn irq_disable() {
	unsafe { asm!("cli" ::: "memory" : "volatile") };
}

/// Determines, if the interrupt flags (IF) is set
#[inline(always)]
pub fn get_rflags() -> u64{
	let rflags: u64;

	unsafe { asm!("pushf; pop $0": "=r"(rflags) :: "memory" : "volatile") };

	rflags
}

/// Determines, if the interrupt flags (IF) is set
#[inline(always)]
pub fn is_irq_enabled() -> bool
{
	let rflags: u64 = get_rflags();

	if (rflags & (1u64 << 9)) !=  0 {
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
#[inline(always)]
pub fn irq_nested_disable() -> bool {
	let was_enabled = is_irq_enabled();
	irq_disable();
	was_enabled
}

/// Enable IRQs (nested)
///
/// Can be used in conjunction with irq_nested_disable() to only enable
/// interrupts again if they were enabled before.
#[inline(always)]
pub fn irq_nested_enable(was_enabled: bool) {
	if was_enabled == true {
		irq_enable();
	}
}

#[no_mangle]
pub extern "C" fn irq_handler(state: *const State) {
	let int_no = unsafe { (*state).int_no };

	//debug!("Task {} receive interrupt {} (rflags 0x{:x})!", get_current_taskid(), int_no,
	//	get_rflags());

	/*
	* If the IDT entry that was invoked was greater-than-or-equal to 40
	* and lower than 48 (meaning IRQ8 - 15), then we need to
	* send an EOI to the slave controller of the PIC
	*/
	if int_no >= 40 {
		unsafe { outb(0x20, 0xA0); }
	}

	/*
	* In either case, we need to send an EOI to the master
	* interrupt controller of the PIC, too
	*/
	unsafe { outb(0x20, 0x20); }

	// if we handle a timer interrupt, we have to trigger the scheduler
	if int_no == 32 {
		schedule();
	}
}
