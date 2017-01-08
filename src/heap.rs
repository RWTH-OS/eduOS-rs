//! Configuration of our system allocator.
//!
//! There's a good chance that `HEAP_BOTTOM` and `HEAP_TOP` stuff involves
//! undefined behavior and thus nasal demons as far as `rustc` is
//! concerned.

use alloc_buddy_simple::{FreeBlock, initialize_allocator};

extern {
    /// The bottom of our heap.  Declared in `boot.asm` so that we can
    /// easily specify alignment constraints.  We declare this as a single
    /// variable of type `u8`, because that's how we get it to link, but we
    /// only want to take the address of it.
    static mut HEAP_BOTTOM: u8;

    /// The top of our heap.  This is actually "one beyond" the heap space,
    /// so storing things here would be Very Bad.  Even just declaring this
    /// probably invokes undefined behavior, but our fingers are crossed.
    static mut HEAP_TOP: u8;
}

/// An array of free lists which we pass to the system allocator at system
/// startup time.
static mut FREE_LISTS: [*mut FreeBlock; 19] = [0 as *mut _; 19];

/// Initialze our system heap.  Once this is done, it's theoretically safe
/// to use functions in libcollection that allocate memory.
pub unsafe fn initialize() {
    // Convert our fake variables into the pointers we wanted in the first
    // place.  Again, there may be some risk of undefined behavior here.
    let heap_bottom_ptr = &mut HEAP_BOTTOM as *mut _;
    let heap_top_ptr = &mut HEAP_TOP as *mut _;

    // Initialize our main allocator library.
    let heap_size = heap_top_ptr as usize - heap_bottom_ptr as usize;
    initialize_allocator(heap_bottom_ptr, heap_size, &mut FREE_LISTS);
}
