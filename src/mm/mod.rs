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

mod hole;
mod linked_list_allocator;
mod freelist;
pub mod page_allocator;

static HEAP: Spinlock<Option<Heap>> = Spinlock::new(None);

pub unsafe fn init(offset: usize, size: usize) {
	info!("Initialize heap at [0x{:x} - 0x{:x}]", offset, offset + size);
    *HEAP.lock() = Some(Heap::new(offset, size));
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

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
#[inline(always)]
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
#[inline(always)]
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
