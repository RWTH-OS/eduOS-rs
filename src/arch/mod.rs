// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::kernel::{serial,processor,irq,init,jump_to_user_land,register_task,
	get_memory_size,get_memfile};

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::load_application;

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::mm;

// Export our platform-specific modules.
#[cfg(target_arch="x86_64")]
pub use self::x86_64::mm::paging::{get_kernel_root_page_table,drop_user_space,PageSize,BasePageSize};

// Implementations for x86_64.
#[cfg(target_arch="x86_64")]
pub mod x86_64;
