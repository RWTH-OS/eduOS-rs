// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use crate::arch::x86_64::mm::paging::page_fault_handler;
use crate::logging::*;
use crate::scheduler::*;
use crate::synch::spinlock::*;
use core::arch::asm;
use core::fmt;
use x86::bits64::paging::VAddr;
use x86::dtables::{lidt, DescriptorTablePointer};
use x86::io::*;
use x86::segmentation::{SegmentSelector, SystemDescriptorTypes64};
use x86::Ring;

/// Maximum possible number of interrupts
const IDT_ENTRIES: usize = 256;
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

/// Enable Interrupts
pub fn irq_enable() {
	unsafe { asm!("sti", options(preserves_flags, nomem, nostack)) };
}

/// Disable Interrupts
pub fn irq_disable() {
	unsafe { asm!("cli", options(preserves_flags, nomem, nostack)) };
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

#[inline(always)]
pub fn send_eoi_to_slave() {
	/*
	 * If the IDT entry that was invoked was greater-than-or-equal to 40
	 * and lower than 48 (meaning IRQ8 - 15), then we need to
	 * send an EOI to the slave controller of the PIC
	 */
	unsafe {
		outb(0xA0, 0x20);
	}
}

#[inline(always)]
pub fn send_eoi_to_master() {
	/*
	 * In either case, we need to send an EOI to the master
	 * interrupt controller of the PIC, too
	 */
	unsafe {
		outb(0x20, 0x20);
	}
}

// Create isr entries, where the number after the
// pseudo error code represents following interrupts:
// 0: Divide By Zero Exception
// 1: Debug Exception
// 2: Non Maskable Interrupt Exception
// 3: Int 3 Exception
// 4: INTO Exception
// 5: Out of Bounds Exception
// 6: Invalid Opcode Exception
// 7: Coprocessor Not Available Exception

extern "x86-interrupt" fn divide_by_zero_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Divide By Zero Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn debug_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Debug Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn nmi_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Non Maskable Interrupt Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn int3_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Int 3 Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn int0_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a INT0 Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn out_of_bound_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Out of Bounds Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn invalid_opcode_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Invalid Opcode Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn no_coprocessor_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Coprocessor Not Available Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

// 8: Double Fault Exception (With Error Code!)

extern "x86-interrupt" fn double_fault_exception(
	stack_frame: ExceptionStackFrame,
	error_code: u64,
) {
	info!(
		"Task {} receive a Double Fault Exception: {:#?}, error_code {}",
		get_current_taskid(),
		stack_frame,
		error_code
	);
	send_eoi_to_master();
	abort();
}

// 9: Coprocessor Segment Overrun Exception

extern "x86-interrupt" fn overrun_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Coprocessor Segment Overrun Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

// 10: Bad TSS Exception (With Error Code!)
// 11: Segment Not Present Exception (With Error Code!)
// 12: Stack Fault Exception (With Error Code!)
// 13: General Protection Fault Exception (With Error Code!)
// 14: Page Fault Exception (With Error Code!)

extern "x86-interrupt" fn bad_tss_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
	info!(
		"Task {} receive a Bad TSS Exception: {:#?}, error_code 0x{:x}",
		get_current_taskid(),
		stack_frame,
		error_code
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn not_present_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
	info!(
		"Task {} receive a Segment Not Present Exception: {:#?}, error_code 0x{:x}",
		get_current_taskid(),
		stack_frame,
		error_code
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn stack_fault_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
	info!(
		"Task {} receive a Stack Fault Exception: {:#?}, error_code 0x{:x}",
		get_current_taskid(),
		stack_frame,
		error_code
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn general_protection_exception(
	stack_frame: ExceptionStackFrame,
	error_code: u64,
) {
	info!(
		"Task {} receive a General Protection Exception: {:#?}, error_code 0x{:x}",
		get_current_taskid(),
		stack_frame,
		error_code
	);
	send_eoi_to_master();
	abort();
}

// 15: Reserved Exception
// 16: Floating Point Exception
// 17: Alignment Check Exception
// 18: Machine Check Exception
// 19-31: Reserved

extern "x86-interrupt" fn floating_point_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Floating Point Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn alignment_check_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Alignment Check Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn machine_check_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a Machine Check Exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn reserved_exception(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive a reserved exception: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn unhandled_irq1(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive unknown interrupt: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn unhandled_irq2(stack_frame: ExceptionStackFrame) {
	info!(
		"Task {} receive unknown interrupt: {:#?}",
		get_current_taskid(),
		stack_frame
	);
	send_eoi_to_slave();
	send_eoi_to_master();
	abort();
}

extern "x86-interrupt" fn timer_handler(stack_frame: ExceptionStackFrame) {
	debug!(
		"Task {} receive timer interrupt!\n{:#?}",
		get_current_taskid(),
		stack_frame
	);

	send_eoi_to_master();
	schedule();
}

/// An interrupt gate descriptor.
///
/// See Intel manual 3a for details, specifically section "6.14.1 64-Bit Mode
/// IDT" and "Figure 6-7. 64-Bit IDT Gate Descriptors".
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
	/// Lower 16 bits of ISR.
	pub base_lo: u16,
	/// Segment selector.
	pub selector: SegmentSelector,
	/// This must always be zero.
	pub ist_index: u8,
	/// Flags.
	pub flags: u8,
	/// The upper 48 bits of ISR (the last 16 bits must be zero).
	pub base_hi: u64,
	/// Must be zero.
	pub reserved1: u16,
}

enum Type {
	InterruptGate,
	TrapGate,
}

impl Type {
	pub fn pack(self) -> u8 {
		match self {
			Type::InterruptGate => SystemDescriptorTypes64::InterruptGate as u8,
			Type::TrapGate => SystemDescriptorTypes64::TrapGate as u8,
		}
	}
}

impl IdtEntry {
	/// A "missing" IdtEntry.
	///
	/// If the CPU tries to invoke a missing interrupt, it will instead
	/// send a General Protection fault (13), with the interrupt number and
	/// some other data stored in the error code.
	pub const MISSING: IdtEntry = IdtEntry {
		base_lo: 0,
		selector: SegmentSelector::from_raw(0),
		ist_index: 0,
		flags: 0,
		base_hi: 0,
		reserved1: 0,
	};

	/// Create a new IdtEntry pointing at `handler`, which must be a function
	/// with interrupt calling conventions.  (This must be currently defined in
	/// assembly language.)  The `gdt_code_selector` value must be the offset of
	/// code segment entry in the GDT.
	///
	/// The "Present" flag set, which is the most common case.  If you need
	/// something else, you can construct it manually.
	pub fn new(
		handler: VAddr,
		gdt_code_selector: SegmentSelector,
		dpl: Ring,
		ty: Type,
		ist_index: u8,
	) -> IdtEntry {
		assert!(ist_index < 0b1000);
		IdtEntry {
			base_lo: ((handler.as_usize() as u64) & 0xFFFF) as u16,
			base_hi: handler.as_usize() as u64 >> 16,
			selector: gdt_code_selector,
			ist_index: ist_index,
			flags: dpl as u8 | ty.pack() | (1 << 7),
			reserved1: 0,
		}
	}
}

static INTERRUPT_HANDLER: SpinlockIrqSave<InteruptHandler> =
	SpinlockIrqSave::new(InteruptHandler::new());

struct InteruptHandler {
	/// An Interrupt Descriptor Table which specifies how to respond to each
	/// interrupt.
	idt: [IdtEntry; IDT_ENTRIES],
}

impl InteruptHandler {
	pub const fn new() -> InteruptHandler {
		InteruptHandler {
			idt: [IdtEntry::MISSING; IDT_ENTRIES],
		}
	}

	pub fn add_handler(
		&mut self,
		int_no: usize,
		func: extern "x86-interrupt" fn(ExceptionStackFrame),
	) {
		if int_no < IDT_ENTRIES {
			self.idt[int_no] = IdtEntry::new(
				VAddr::from_usize(func as usize),
				KERNEL_CODE_SELECTOR,
				Ring::Ring0,
				Type::InterruptGate,
				0,
			);
		} else {
			info!("unable to add handler for interrupt {}", int_no);
		}
	}

	pub fn remove_handler(&mut self, int_no: usize) {
		if int_no < IDT_ENTRIES {
			if int_no < 40 {
				self.idt[int_no] = IdtEntry::new(
					VAddr::from_usize(unhandled_irq1 as usize),
					KERNEL_CODE_SELECTOR,
					Ring::Ring0,
					Type::InterruptGate,
					0,
				);
			} else {
				// send  eoi to the master and to the slave
				self.idt[int_no] = IdtEntry::new(
					VAddr::from_usize(unhandled_irq2 as usize),
					KERNEL_CODE_SELECTOR,
					Ring::Ring0,
					Type::InterruptGate,
					0,
				);
			}
		} else {
			info!("unable to remove handler for interrupt {}", int_no);
		}
	}

	pub unsafe fn load_idt(&mut self) {
		self.idt[0] = IdtEntry::new(
			VAddr::from_usize(divide_by_zero_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[1] = IdtEntry::new(
			VAddr::from_usize(debug_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[2] = IdtEntry::new(
			VAddr::from_usize(nmi_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			1,
		);
		self.idt[3] = IdtEntry::new(
			VAddr::from_usize(int3_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[4] = IdtEntry::new(
			VAddr::from_usize(int0_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[5] = IdtEntry::new(
			VAddr::from_usize(out_of_bound_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[6] = IdtEntry::new(
			VAddr::from_usize(invalid_opcode_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[7] = IdtEntry::new(
			VAddr::from_usize(no_coprocessor_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[8] = IdtEntry::new(
			VAddr::from_usize(double_fault_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			2,
		);
		self.idt[9] = IdtEntry::new(
			VAddr::from_usize(overrun_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[10] = IdtEntry::new(
			VAddr::from_usize(bad_tss_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[11] = IdtEntry::new(
			VAddr::from_usize(not_present_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[12] = IdtEntry::new(
			VAddr::from_usize(stack_fault_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[13] = IdtEntry::new(
			VAddr::from_usize(general_protection_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[14] = IdtEntry::new(
			VAddr::from_usize(page_fault_handler as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[15] = IdtEntry::new(
			VAddr::from_usize(reserved_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[16] = IdtEntry::new(
			VAddr::from_usize(floating_point_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[17] = IdtEntry::new(
			VAddr::from_usize(alignment_check_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);
		self.idt[18] = IdtEntry::new(
			VAddr::from_usize(machine_check_exception as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			3,
		);
		for i in 19..32 {
			self.idt[i] = IdtEntry::new(
				VAddr::from_usize(reserved_exception as usize),
				KERNEL_CODE_SELECTOR,
				Ring::Ring0,
				Type::InterruptGate,
				0,
			);
		}
		self.idt[32] = IdtEntry::new(
			VAddr::from_usize(timer_handler as usize),
			KERNEL_CODE_SELECTOR,
			Ring::Ring0,
			Type::InterruptGate,
			0,
		);

		// send only eoi to the master
		for i in 33..40 {
			self.idt[i] = IdtEntry::new(
				VAddr::from_usize(unhandled_irq1 as usize),
				KERNEL_CODE_SELECTOR,
				Ring::Ring0,
				Type::InterruptGate,
				0,
			);
		}
		// send  eoi to the master and to the slave
		for i in 40..IDT_ENTRIES {
			self.idt[i] = IdtEntry::new(
				VAddr::from_usize(unhandled_irq2 as usize),
				KERNEL_CODE_SELECTOR,
				Ring::Ring0,
				Type::InterruptGate,
				0,
			);
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
unsafe fn irq_remap() {
	outb(0x20, 0x11);
	outb(0xA0, 0x11);
	outb(0x21, 0x20);
	outb(0xA1, 0x28);
	outb(0x21, 0x04);
	outb(0xA1, 0x02);
	outb(0x21, 0x01);
	outb(0xA1, 0x01);
	outb(0x21, 0x00);
	outb(0xA1, 0x00);
}

pub fn init() {
	debug!("initialize interrupt descriptor table");

	unsafe {
		irq_remap();

		// load address of the IDT
		INTERRUPT_HANDLER.lock().load_idt();
	}
}

// derived from hilipp Oppermann's blog
// => https://github.com/phil-opp/blog_os/blob/master/src/interrupts/mod.rs

/// Represents the exception stack frame pushed by the CPU on exception entry.
#[repr(C)]
pub struct ExceptionStackFrame {
	/// This value points to the instruction that should be executed when the interrupt
	/// handler returns. For most interrupts, this value points to the instruction immediately
	/// following the last executed instruction. However, for some exceptions (e.g., page faults),
	/// this value points to the faulting instruction, so that the instruction is restarted on
	/// return. See the documentation of the `Idt` fields for more details.
	pub instruction_pointer: u64,
	/// The code segment selector, padded with zeros.
	pub code_segment: u64,
	/// The flags register before the interrupt handler was invoked.
	pub cpu_flags: u64,
	/// The stack pointer at the time of the interrupt.
	pub stack_pointer: u64,
	/// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
	pub stack_segment: u64,
}

impl fmt::Debug for ExceptionStackFrame {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		struct Hex(u64);
		impl fmt::Debug for Hex {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "{:#x}", self.0)
			}
		}

		let mut s = f.debug_struct("ExceptionStackFrame");
		s.field("instruction_pointer", &Hex(self.instruction_pointer));
		s.field("code_segment", &Hex(self.code_segment));
		s.field("cpu_flags", &Hex(self.cpu_flags));
		s.field("stack_pointer", &Hex(self.stack_pointer));
		s.field("stack_segment", &Hex(self.stack_segment));
		s.finish()
	}
}
