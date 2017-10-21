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

//! Interface to initialize the processor and to detect CPU features

use cpuio;
use x86::shared::*;
use logging::*;

pub fn halt() {
	loop {
		unsafe {
			asm!("hlt" :::: "volatile");
		}
	}
}

#[inline(always)]
pub fn pause() {
	unsafe {
		asm!("pause" :::: "volatile");
	}
}

pub fn shutdown() {
	// shutdown, works only on Qemu
	unsafe {
		let mut shutdown_port : cpuio::Port<u8> = cpuio::Port::new(0xf4);
		shutdown_port.write(0x00);
	};

	halt();
}

pub fn init() {
	info!("enable supported processor features");

	let mut cr0 = unsafe { control_regs::cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | control_regs::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | control_regs::CR0_NUMERIC_ERROR;
	cr0 = cr0 | control_regs::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(control_regs::CR0_CACHE_DISABLE|control_regs::CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { control_regs::cr0_write(cr0) };
}
