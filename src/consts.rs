// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(dead_code)]

//! Configuration parameter of the kernel eduOS-rs

/// Define the size of the kernel stack
pub const STACK_SIZE : usize = 0x2000;

/// Size of a cache line
pub const CACHE_LINE : usize = 64;

/// Size of a page frame on a x86_64 processor
#[cfg(target_arch="x86_64")]
pub const PAGE_SIZE : usize = 4096;
