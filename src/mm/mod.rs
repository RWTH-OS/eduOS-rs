// derived from Philipp Oppermann's memory allocator
//
// The original crate is dual-licensed under MIT or the Apache License (Version 2.0).
// See LICENSE-APACHE and LICENSE-MIT for details.
//
// original repository = "https://github.com/phil-opp/linked-list-allocator"
// documentation = "https://docs.rs/crate/linked_list_allocator"
// homepage = "http://os.phil-opp.com/kernel-heap.html#a-better-allocator"
//
// November 2017: adapted for eduOS-rs by Stefan Lankes

use alloc::heap::{Alloc, AllocErr, Layout};
use synch::spinlock::Spinlock;
use self::linked_list_allocator::Heap;
use logging::*;
use arch::paging;

mod hole;
mod linked_list_allocator;
mod freelist;
pub mod page_allocator;
pub mod vma;

extern "C" {
	/// entry point of the kernel and defined by the linker script
	static kernel_start: u8;

	/// end point of the kernel and defined by the linker script
	static kernel_end: u8;
}

static HEAP: Spinlock<Option<Heap>> = Spinlock::new(None);

pub unsafe fn init(offset: usize, size: usize) {
	info!("Initialize heap at [0x{:x} - 0x{:x}]", offset, offset + size);
    *HEAP.lock() = Some(Heap::new(offset, size));
	vma::vma_add(offset, size, vma::VmaType::READ | vma::VmaType::WRITE |
		vma::VmaType::EXECUTE | vma::VmaType::CACHEABLE);

	paging::init();
}

pub fn kernel_start_address() -> usize {
	unsafe { align_down!(&kernel_start as *const u8 as usize, 0x200000) }
}

pub fn kernel_end_address() -> usize {
	unsafe { align_up!(&kernel_end as *const u8 as usize, 0x200000) }
}

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout)
        } else {
            panic!("__rust_allocate: heap not initialized");
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout)
        } else {
            panic!("__rust_deallocate: heap not initialized");
        }
    }
}
