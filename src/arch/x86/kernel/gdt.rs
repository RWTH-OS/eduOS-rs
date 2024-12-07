use crate::arch::mm::VirtAddr;
use crate::consts::*;
use crate::scheduler;
use core::mem;
use x86::bits64::segmentation::*;
use x86::bits64::task::*;
use x86::controlregs::cr3_write;
use x86::dtables::{self, DescriptorTablePointer};
use x86::segmentation::*;
use x86::Ring;

const GDT_NULL: usize = 0;
const GDT_KERNEL_CODE: usize = 1;
const GDT_KERNEL_DATA: usize = 2;
const GDT_USER32_CODE: usize = 3;
const GDT_USER32_DATA: usize = 4;
#[cfg(target_arch = "x86_64")]
const GDT_USER64_CODE: usize = 5;
#[cfg(target_arch = "x86_64")]
const GDT_FIRST_TSS: usize = 6;
#[cfg(target_arch = "x86")]
const GDT_FIRST_TSS: usize = 5;

// fox x86_64 is a TSS descriptor twice larger than a code/data descriptor
#[cfg(target_arch = "x86_64")]
const TSS_ENTRIES: usize = 2;
#[cfg(target_arch = "x86")]
const TSS_ENTRIES: usize = 1;
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
pub(crate) struct Tss(TaskStateSegment);

impl Tss {
	#[allow(dead_code)]
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
pub(crate) fn init() {
	#[cfg(target_arch = "x86_64")]
	let limit = 0;
	#[cfg(target_arch = "x86")]
	let limit = 0xFFFF_FFFF;

	unsafe {
		// The NULL descriptor is always the first entry.
		GDT[GDT_NULL] = Descriptor::NULL;

		#[cfg(target_arch = "x86_64")]
		{
			// The second entry is a 64bit Code Segment in kernel-space (Ring 0).
			// All other parameters are ignored.
			GDT[GDT_KERNEL_CODE] =
				DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
					.present()
					.dpl(Ring::Ring0)
					.l()
					.finish();
		}
		#[cfg(target_arch = "x86")]
		{
			// The second entry is a 32bit Code Segment in kernel-space (Ring 0).
			// All other parameters are ignored.
			GDT[GDT_KERNEL_CODE] =
				DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
					.present()
					.dpl(Ring::Ring0)
					.db()
					.limit_granularity_4kb()
					.finish();
		}

		// The third entry is a Data Segment in kernel-space (Ring 0).
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

		#[cfg(target_arch = "x86_64")]
		{
			/*
			 * Create code segment for 64bit user-space applications (ring 3)
			 */
			GDT[GDT_USER64_CODE] =
				DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
					.present()
					.dpl(Ring::Ring3)
					.l()
					.finish();
		}

		/*
		 * Create TSS for each core (we use these segments for task switching)
		 */
		#[cfg(target_arch = "x86_64")]
		{
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
		}
		#[cfg(target_arch = "x86")]
		{
			let base = &TSS.0 as *const _ as u64;
			let tss_descriptor: Descriptor =
				<DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(
					base,
					base + mem::size_of::<TaskStateSegment>() as u64 - 1,
					true,
				)
				.present()
				.dpl(Ring::Ring0)
				.finish();

			/* set default values */
			TSS.0.eflags = 0x1202;
			TSS.0.ss0 = 0x10; // data segment
			TSS.0.esp0 = get_boot_stack().interrupt_top().into();
			TSS.0.cs = 0x0b;

			GDT[GDT_FIRST_TSS] = tss_descriptor;
		}

		// load GDT
		let gdtr = DescriptorTablePointer::new(&GDT);
		dtables::lgdt(&gdtr);

		// Reload the segment descriptors
		load_cs(SegmentSelector::new(GDT_KERNEL_CODE as u16, Ring::Ring0));
		load_ss(SegmentSelector::new(GDT_KERNEL_DATA as u16, Ring::Ring0));
	}
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
	TSS.0.rsp[0] = stack.as_u64();
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
	TSS.0.esp = stack.as_u32();
}

#[no_mangle]
pub(crate) unsafe extern "C" fn set_current_kernel_stack() {
	cr3_write(scheduler::get_root_page_table().as_u64());

	let rsp = scheduler::get_current_stack();
	set_kernel_stack(rsp + STACK_SIZE - 0x10u64);
}
