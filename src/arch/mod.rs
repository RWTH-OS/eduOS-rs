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

#![allow(dead_code)]

//! Architecture dependent interface

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::{serial,processor,irq,pit,gdt};

// Implementations for x86_64.
#[cfg(target_arch="x86_64")]
#[macro_use]
pub mod x86_64;

#[cfg(target_arch="x86_64")]
const PAGE_SIZE: u64 = 4096;

use mm;
use mm::{align_up, align_down};
use multiboot::{Multiboot, MemoryType, PAddr};
use core::slice;
use core::mem;
use logging::*;
use consts::*;
use mm::page_allocator::add_region;
use mm::vma::{vma_dump, vma_add, VmaType};
use x86::shared::task::load_tr;
use x86::shared::segmentation::SegmentSelector;
use x86::shared::PrivilegeLevel;

extern {
	/// Begin of the kernel.  Declared in `linker.ld` so that we can
	/// easily specify alignment constraints.  We declare this as a single
	/// variable of type `u8`, because that's how we get it to link, but we
	/// only want to take the address of it.
	static mut kernel_start: u8;

	/// End of the kernel.  Declared in `linker.ld` so that we can
	/// easily specify alignment constraints.  We declare this as a single
	/// variable of type `u8`, because that's how we get it to link, but we
	/// only want to take the address of it.
	static mut kernel_end: u8;

	/// Pointer to the multiboot info, declared in `boot.asm`.
	static MBINFO: u32;
}

fn paddr_to_slice<'a>(p: PAddr, sz: usize) -> Option<&'a [u8]> {
	unsafe {
		let ptr = mem::transmute(p as u64);
		Some(slice::from_raw_parts(ptr, sz))
	}
}

fn initialize_memory() {
	unsafe {
		let kernel_end_ptr = &mut kernel_end as *mut _;
		let kernel_start_ptr = &mut kernel_start as *mut _;
		let kernel_end_usize = align_up(kernel_end_ptr as usize, PAGE_SIZE as usize);
		let kernel_start_usize = align_up(kernel_start_ptr as usize, PAGE_SIZE as usize);
		let mut total: u64 = 0;
		let mb = Multiboot::new(MBINFO as PAddr, paddr_to_slice);

		// start heap directly after the kernel
		mm::init(kernel_end_usize as usize, align_up(kernel_end_usize, 0x200000usize) - kernel_end_usize);
		vma_add(align_down(MBINFO as usize, PAGE_SIZE as usize), PAGE_SIZE as usize, VmaType::READ);

		mb.as_ref().unwrap().memory_regions().map(|regions| {
			for region in regions {
				if region.memory_type() == MemoryType::Available {
					let mut base = region.base_address();
					let mut len = region.length();

					total += len;

					if  region.base_address() < kernel_start_usize as u64
					 && region.base_address() + region.length() >= kernel_start_usize as u64 {
						// regions below 1M are reserved for IO devices
						add_region(0x100000, kernel_start_usize - 0x100000);
					}

					if base < (kernel_end_usize as u64) && base + len > (kernel_end_usize as u64) {
						len = len - (align_up(kernel_end_usize, 0x200000usize) as u64 - base);
						base = align_up(kernel_end_usize, 0x200000usize) as u64;
					}

					if base > 0x100000 {
						add_region(base as usize, len as usize);
					}
				}
			}
		});

		vma_dump();
		info!("Current allocated memory: {} KiB", (kernel_end_usize - kernel_start_usize) >> 10);
		info!("Current available memory: {} MiB", total >> 20);
	}
}

extern {
	pub fn __replace_boot_stack(stack_bottom: usize);
}

/// The boot loader initialize a stack, which is later also required to
/// to boot other core. Consequently, the kernel has to replace with this
/// function the boot stack by a new one.
pub fn replace_boot_stack(stack_bottom: usize, ist_bottom: usize)
{
	unsafe {
		__replace_boot_stack(stack_bottom);

		gdt::set_kernel_stack(stack_bottom + KERNEL_STACK_SIZE - 0x10,
			ist_bottom + KERNEL_STACK_SIZE - 0x10);

		// register task
		let sel = SegmentSelector::new(gdt::GDT_FIRST_TSS as u16, PrivilegeLevel::Ring0);
		load_tr(sel);
	}
}

pub fn jump_to_user_land(func: fn() -> !) -> !
{
	let ds = 0x23u64;
	let cs = 0x2bu64;

	unsafe {
		asm!("mov $0, %ds; mov $0, %es; push $0; push %rsp; addq $$16, (%rsp); pushfq; push $1; push $2; iretq"
			:: "r"(ds), "r"(cs), "r"(func as u64)
			:: "volatile");
	}

	loop {
		processor::halt();
	}
}

/// Initialize module, must be called once, and only once
pub fn init() {
	serial::init();
	processor::init();
	initialize_memory();
	gdt::init();
	irq::init();
	pit::init();
}
