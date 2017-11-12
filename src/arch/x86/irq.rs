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
use synch::spinlock::*;
use cpuio::outb;
use arch::x86::task::State;
use x86::shared::dtables::{DescriptorTablePointer,lidt};
use x86::shared::PrivilegeLevel;
use x86::shared::paging::VAddr;
#[cfg(target_arch="x86_64")]
use x86::bits64::irq::{IdtEntry, Type};
#[cfg(target_arch="x86")]
use x86::bits32::irq::IdtEntry;
use x86::shared::segmentation::SegmentSelector;

/// Maximum possible number of interrupts
const IDT_ENTRIES: usize = 256;
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);

/// This is a simple string array. It contains the message that
/// corresponds to each and every exception. We get the correct
/// message by accessing it like this:
/// exception_message[interrupt_number]
const EXCEPTION_MESSAGES: [&'static str; 32] = [
	"Division By Zero", "Debug", "Non Maskable Interrupt",
	"Breakpoint", "Into Detected Overflow", "Out of Bounds", "Invalid Opcode",
	"No Coprocessor", "Double Fault", "Coprocessor Segment Overrun", "Bad TSS",
	"Segment Not Present", "Stack Fault", "General Protection Fault", "Page Fault",
	"Unknown Interrupt", "Coprocessor Fault", "Alignment Check", "Machine Check",
	"SIMD Floating-Point", "Virtualization", "Reserved", "Reserved", "Reserved",
	"Reserved", "Reserved", "Reserved", "Reserved", "Reserved", "Reserved",
	"Reserved", "Reserved"];

#[allow(dead_code)]
extern {
	/// Interrupt handlers => see irq_handler.asm
	static basic_interrupt_handlers: [*const u8; IDT_ENTRIES];
}

unsafe fn unhandled_irq(state: *const State)
{
	info!("receqive unhandled interrupt {}", (*state).int_no);
}

/// All of our Exception handling Interrupt Service Routines will
/// point to this function. This will tell us what exception has
/// occured! Right now, we simply abort the current task.
/// All ISRs disable interrupts while they are being
/// serviced as a 'locking' mechanism to prevent an IRQ from
/// happening and messing up kernel data structures
unsafe fn fault_handler(state: *const State)
{
	let int_no = (*state).int_no;

	if int_no < 32 {
		info!("{} Exception ({}) at 0x{:x}:0x{:x}, error code 0x{:x}, eflags 0x{:x}",
			EXCEPTION_MESSAGES[int_no as usize], int_no, (*state).cs, (*state).ip,
			(*state).error, (*state).eflags);

		outb(0x20, 0x20);

		irq_enable();
		abort();
	}
}

unsafe fn timer_handler(_state: *const State)
{
	// nothing to do
}

static INTERRUPT_HANDLER: SpinlockIrqSave<InteruptHandler> = SpinlockIrqSave::new(InteruptHandler::new());

struct InteruptHandler {
	/// An Interrupt Descriptor Table which specifies how to respond to each
	/// interrupt.
	idt: [IdtEntry; IDT_ENTRIES],
	irq_handler: [unsafe fn(state: *const State); IDT_ENTRIES]
}

impl InteruptHandler {
	pub const fn new() -> InteruptHandler {
		InteruptHandler {
			idt: [IdtEntry::MISSING; IDT_ENTRIES],
			irq_handler: [unhandled_irq; IDT_ENTRIES]
		}
	}

	pub fn get_handler(&mut self, int_no: usize) -> unsafe fn(state: *const State)
	{
		self.irq_handler[int_no]
	}

	pub fn add_handler(&mut self, int_no: usize, func: unsafe fn(state: *const State))
	{
		if int_no < IDT_ENTRIES {
			self.irq_handler[int_no] = func;
		} else {
			info!("unable to add handler for interrupt {}", int_no);
		}
	}

	pub fn remove_handler(&mut self, int_no: usize)
	{
		if int_no < IDT_ENTRIES {
			self.irq_handler[int_no] = unhandled_irq;
		} else {
			info!("unable to remove handler for interrupt {}", int_no);
		}
	}

	pub unsafe fn load_idt(&mut self) {
		for i in 0..IDT_ENTRIES {
			self.idt[i] = IdtEntry::new(VAddr::from_usize(basic_interrupt_handlers[i] as usize),
				KERNEL_CODE_SELECTOR, PrivilegeLevel::Ring0, Type::InterruptGate, 0);
		}

		let idtr = DescriptorTablePointer::new(&self.idt);
		lidt(&idtr);
	}
}

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

	unsafe { irq_remap(); }

	let mut guard = INTERRUPT_HANDLER.lock();

	// all exceptions will be handled by fault_handler
	for i in 0..32 {
		guard.add_handler(i, fault_handler);
	}

	// dummy handler for timer interrsupts
	guard.add_handler(32, timer_handler);

	// load address of the IDT
	unsafe { guard.load_idt(); }
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

/// Determines the value of the status register
#[inline(always)]
pub fn get_eflags() -> usize{
	let eflags: usize;

	unsafe { asm!("pushf; pop $0": "=r"(eflags) :: "memory" : "volatile") };

	eflags
}

/// Determines, if the interrupt flag (IF) is set
#[inline(always)]
pub fn is_irq_enabled() -> bool
{
	let eflags: usize = get_eflags();

	if (eflags & (1usize << 9)) !=  0 {
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

/// Each of the IRQ ISRs point to this function, rather than
/// the 'fault_handler' in 'isrs.c'. The IRQ Controllers need
/// to be told when you are done servicing them, so you need
/// to send them an "End of Interrupt" command. If we use the PIC
/// instead of the APIC, we have two 8259 chips: The first one
/// exists at 0x20, the second one exists at 0xA0. If the second
/// controller (an IRQ from 8 to 15) gets an interrupt, you need to
/// acknowledge the interrupt at BOTH controllers, otherwise, you
/// only send an EOI command to the first controller. If you don't send
/// an EOI, it won't raise any more IRQs.
///
/// Note: If we enabled the APIC, we also disabled the PIC. Afterwards,
/// we get no interrupts between 0 and 15.
#[no_mangle]
pub extern "C" fn irq_handler(state: *const State) {
	let int_no = unsafe { (*state).int_no };

	debug!("Task {} receive interrupt {} (eflags 0x{:x})!", get_current_taskid(), int_no,
		get_eflags());

	let handler = INTERRUPT_HANDLER.lock().get_handler(int_no as usize);
	unsafe { handler(state); }

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
