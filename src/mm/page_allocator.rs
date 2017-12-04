// Copyright (c) 2017 Colin Finck, RWTH Aachen University
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

use logging::*;
use consts::*;
use mm::freelist::{FreeList, FreeListEntry};

static PHYSICAL_FREE_LIST: FreeList = FreeList::new();

pub fn add_region(start: usize, len: usize)
{
	info!("add free memory region [0x{:08x} - 0x{:08x}] to the page allocator", start, start+len);

	let entry = FreeListEntry {
		start: start as usize,
		end: len as usize
	};

	PHYSICAL_FREE_LIST.list.lock().push(entry);
}

pub fn allocate(size: usize) -> usize {
	assert!(size & (PAGE_SIZE - 1) == 0, "Size {:#X} is not aligned to {:#X}", size, PAGE_SIZE);

	let result = PHYSICAL_FREE_LIST.allocate(size);
	assert!(result.is_ok(), "Could not allocate {:#X} bytes of physical memory", size);
	result.unwrap()
}

pub fn deallocate(physical_address: usize, size: usize) {
	assert!(size & (PAGE_SIZE - 1) == 0, "Size {:#X} is not aligned to {:#X}", size, PAGE_SIZE);

	PHYSICAL_FREE_LIST.deallocate(physical_address, size);
}
