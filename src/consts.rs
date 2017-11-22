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

//! Configuration parameter of the kernel eduOS-rs

/// Define the size of the kernel stack
pub const KERNEL_STACK_SIZE : usize = 8192;

/// Size of a cache line on a x86_64 processor
#[cfg(target_arch="x86_64")]
pub const CACHE_LINE : usize = 64;

/// Size of a page frame on a x86_64 processor
#[cfg(target_arch="x86_64")]
pub const PAGE_SIZE : usize = 4096;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 32;

/// penalty for multilevel feedback
pub const SCHEDULING_PENALTY: u8 = 8;

/// frequency of the timer interrupt
pub const TIMER_FREQ: u32 = 100; /* in HZ */
