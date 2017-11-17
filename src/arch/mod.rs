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
pub mod x86_64;

#[cfg(target_arch="x86_64")]
const PAGE_SIZE: u64 = 4096;

use allocator;
use multiboot::{Multiboot, MemoryType, PAddr};
use core::slice;
use core::mem;
use logging::*;
use consts::*;
use x86::shared::task::load_tr;
use x86::shared::segmentation::SegmentSelector;

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
		let kernel_end_u64 = (kernel_end_ptr as u64 + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
		let kernel_start_u64 = (kernel_start_ptr as u64 + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
		let mut init_heap: bool = false;
		let mut total: u64 = 0;
		let mb = Multiboot::new(MBINFO as PAddr, paddr_to_slice);

		mb.as_ref().unwrap().memory_regions().map(|regions| {
			for region in regions {
				if region.memory_type() == MemoryType::Available {
					let mut base = region.base_address();
					let mut len = region.length();

					total += len;

					if base < kernel_end_u64 && base + len > kernel_end_u64 {
						len = len - (kernel_end_u64 - base);
						base = kernel_end_u64;
					}

					// use only memory, which is located above the kernel
					if init_heap == false && base >= kernel_end_u64 {
						info!("Heap starts at 0x{:x} with a size of {} MBytes",
							base, len / (1024*1024));
						init_heap = true;
						allocator::init(base as usize, len as usize);
					}
				}
			}
		});

		info!("Current allocated memory: {} KiB", (kernel_end_u64 - kernel_start_u64) / 1024);
		info!("Current available memory: {} MiB", total / (1024*1024));
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
		let sel: u16 = 3 << 3;
		load_tr(SegmentSelector::from_raw(sel));
	}
}

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	initialize_memory();
	gdt::init();
	irq::init();
	pit::init();
}
