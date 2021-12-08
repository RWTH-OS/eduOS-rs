// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use crate::consts::*;
use core::mem;
use x86::bits64::segmentation::*;
use x86::bits64::task::*;
use x86::controlregs::{cr3, cr3_write};
use x86::dtables::{self, DescriptorTablePointer};
use x86::segmentation::*;
use x86::Ring;
//use logging::*;
use crate::scheduler;

const GDT_NULL: usize = 0;
const GDT_KERNEL_CODE: usize = 1;
const GDT_KERNEL_DATA: usize = 2;
const GDT_USER32_CODE: usize = 3;
const GDT_USER32_DATA: usize = 4;
const GDT_USER64_CODE: usize = 5;
const GDT_FIRST_TSS: usize = 6;

// fox x86_64 is a TSS descriptor twice larger than a code/data descriptor
const TSS_ENTRIES: usize = 2;
const GDT_ENTRIES: usize = GDT_FIRST_TSS + TSS_ENTRIES;

/// We use IST1 through IST4.
/// Each critical exception (NMI, Double Fault, Machine Check) gets a dedicated one while IST1 is shared for all other
/// interrupts. See also irq.rs.
const IST_ENTRIES: usize = 4;

// thread_local on a static mut, signals that the value of this static may
// change depending on the current thread.
static mut GDT: [Descriptor; GDT_ENTRIES] = [Descriptor::NULL; GDT_ENTRIES];
static mut TSS: Tss = Tss::from(TaskStateSegment::new());
static IST: [u8; IST_ENTRIES * STACK_SIZE] = [0; IST_ENTRIES * STACK_SIZE];

// workaround to use the new repr(align) feature
// currently, it is only supported by structs
// => map all task state segments in a struct
#[repr(align(128))]
pub struct Tss(TaskStateSegment);

impl Tss {
	pub const fn into(self) -> TaskStateSegment {
		self.0
	}

	pub const fn from(x: TaskStateSegment) -> Self {
		Tss(x)
	}
}

/// This will setup the special GDT
/// pointer, set up the entries in our GDT, and then
/// finally to load the new GDT and to update the
/// new segment registers
pub fn init() {
	unsafe {
		// The NULL descriptor is always the first entry.
		GDT[GDT_NULL] = Descriptor::NULL;

		// The second entry is a 64-bit Code Segment in kernel-space (Ring 0).
		// All other parameters are ignored.
		GDT[GDT_KERNEL_CODE] =
			DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
				.present()
				.dpl(Ring::Ring0)
				.l()
				.finish();

		// The third entry is a 64-bit Data Segment in kernel-space (Ring 0).
		// All other parameters are ignored.
		GDT[GDT_KERNEL_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
			.present()
			.dpl(Ring::Ring0)
			.finish();

		/*
		 * Create code segment for 32bit user-space applications (ring 3)
		 */
		GDT[GDT_USER32_CODE] =
			DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
				.present()
				.dpl(Ring::Ring3)
				.finish();

		/*
		 * Create code segment for 32bit user-space applications (ring 3)
		 */
		GDT[GDT_USER32_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
			.present()
			.dpl(Ring::Ring3)
			.finish();

		/*
		 * Create code segment for 64bit user-space applications (ring 3)
		 */
		GDT[GDT_USER64_CODE] =
			DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
				.present()
				.dpl(Ring::Ring3)
				.l()
				.finish();

		/*
		 * Create TSS for each core (we use these segments for task switching)
		 */
		let base = &TSS.0 as *const _ as u64;
		let tss_descriptor: Descriptor64 =
			<DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(
				base,
				mem::size_of::<TaskStateSegment>() as u64 - 1,
				true,
			)
			.present()
			.dpl(Ring::Ring0)
			.finish();
		GDT[GDT_FIRST_TSS..GDT_FIRST_TSS + TSS_ENTRIES]
			.copy_from_slice(&mem::transmute::<Descriptor64, [Descriptor; 2]>(
				tss_descriptor,
			));

		// Allocate all ISTs for this core.
		for i in 0..IST_ENTRIES {
			TSS.0.ist[i] = &IST[i * STACK_SIZE] as *const _ as u64 + STACK_SIZE as u64 - 0x10;
		}

		// load GDT
		let gdtr = DescriptorTablePointer::new(&GDT);
		dtables::lgdt(&gdtr);

		// Reload the segment descriptors
		load_cs(SegmentSelector::new(GDT_KERNEL_CODE as u16, Ring::Ring0));
		load_ss(SegmentSelector::new(GDT_KERNEL_DATA as u16, Ring::Ring0));
	}
}

#[inline(always)]
unsafe fn set_kernel_stack(stack: usize) {
	TSS.0.rsp[0] = stack as u64;
}

#[no_mangle]
pub unsafe extern "C" fn set_current_kernel_stack() {
	let root = scheduler::get_root_page_table() as u64;
	if root != cr3() {
		cr3_write(root);
	}

	let rsp = scheduler::get_current_stack();
	set_kernel_stack(rsp + STACK_SIZE - 0x10);
}
