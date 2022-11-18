// Copyright (c) 2017-2022 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

// Implementations for x86_64.
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::kernel::{processor, serial};

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::kernel::switch::switch;

// Export our platform-specific modules.
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::mm;
