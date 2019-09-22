// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

use consts::*;
use core::mem;
use x86::bits64::segmentation::*;
use x86::bits64::task::*;
use x86::dtables::{self, DescriptorTablePointer};
use x86::segmentation::*;
use x86::Ring;
//use logging::*;
use scheduler;

const GDT_NULL: usize = 0;
const GDT_KERNEL_CODE: usize = 1;
const GDT_KERNEL_DATA: usize = 2;
const GDT_FIRST_TSS: usize = 3;

// fox x86_64 is a TSS descriptor twice larger than a code/data descriptor
const TSS_ENTRIES: usize = 2;
const GDT_ENTRIES: usize = (3 + TSS_ENTRIES);

// thread_local on a static mut, signals that the value of this static may
// change depending on the current thread.
static mut GDT: [Descriptor; GDT_ENTRIES] = [Descriptor::NULL; GDT_ENTRIES];
static mut TSS: Tss = Tss::from(TaskStateSegment::new());

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
		 * Create TSS for each core (we use these segments for task switching)
		 */
		let base = &TSS.0 as *const _ as u64;
		let tss_descriptor: Descriptor64 =
			<DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(
				base,
				base + mem::size_of::<TaskStateSegment>() as u64 - 1,
				true,
			)
			.present()
			.dpl(Ring::Ring0)
			.finish();
		GDT[GDT_FIRST_TSS..GDT_FIRST_TSS + TSS_ENTRIES]
			.copy_from_slice(&mem::transmute::<Descriptor64, [Descriptor; 2]>(
				tss_descriptor,
			));

		// load GDT
		let gdtr = DescriptorTablePointer::new(&GDT);
		dtables::lgdt(&gdtr);

		// Reload the segment descriptors
		load_cs(SegmentSelector::new(GDT_KERNEL_CODE as u16, Ring::Ring0));
		load_ds(SegmentSelector::new(GDT_KERNEL_DATA as u16, Ring::Ring0));
		load_es(SegmentSelector::new(GDT_KERNEL_DATA as u16, Ring::Ring0));
		load_ss(SegmentSelector::new(GDT_KERNEL_DATA as u16, Ring::Ring0));
	}
}

#[inline(always)]
pub unsafe fn set_kernel_stack(stack: usize) {
	TSS.0.rsp[0] = stack as u64;
}

#[no_mangle]
pub unsafe extern "C" fn set_current_kernel_stack() {
	set_kernel_stack(scheduler::get_current_stack() + STACK_SIZE - 0x10);
}
