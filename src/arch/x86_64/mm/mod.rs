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

pub mod paging;
pub mod physicalmem;
pub mod virtualmem;

use arch::x86_64::kernel::get_memfile;
use self::paging::{PageSize,BasePageSize,PageTableEntryFlags};

pub fn init() {
	paging::init();
	physicalmem::init();
	virtualmem::init();

	let (start, len) = get_memfile();
	if len > 0 {
		// Map file into the kernel space
		paging::map::<BasePageSize>(align_down!(start as usize, BasePageSize::SIZE),
			align_down!(start as usize, BasePageSize::SIZE),
			align_up!(len as usize, BasePageSize::SIZE)/BasePageSize::SIZE,
			PageTableEntryFlags::GLOBAL|PageTableEntryFlags::EXECUTE_DISABLE);

		virtualmem::reserve(align_down!(start as usize, BasePageSize::SIZE),
			align_up!(len as usize, BasePageSize::SIZE));
	}
}
