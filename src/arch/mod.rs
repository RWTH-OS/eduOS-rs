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

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::{serial,processor};

// Implementations for x86_64.
#[cfg(target_arch="x86_64")]
pub mod x86_64;

use allocator;

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

/// Init memory module
/// Must be called once, and only once
pub fn init() {
	processor::init();

	unsafe {
		let heap_bottom_ptr = &mut HEAP_BOTTOM as *mut _;
		let heap_top_ptr = &mut HEAP_TOP as *mut _;
		let heap_size = heap_top_ptr as usize - heap_bottom_ptr as usize;

		// Initialize allocator
		allocator::init(heap_bottom_ptr as usize, heap_size);
	}
}
