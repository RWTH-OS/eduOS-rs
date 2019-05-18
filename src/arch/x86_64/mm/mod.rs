// Copyright (c) 2017 Colin Finck, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
