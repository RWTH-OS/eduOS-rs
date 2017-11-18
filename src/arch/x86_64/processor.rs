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
use raw_cpuid::*;
use x86::shared::*;
use x86::shared::control_regs::*;
use logging::*;
use consts::*;
use timer::*;
use core::sync::atomic::hint_core_should_pause;
use x86::shared::time::rdtsc;

lazy_static! {
	static ref FREQUENCY: u32 = {
		let old = TIMER.get_clock_tick();
		let mut ticks = old;
		let diff: u64;

		/* wait for the next time slice */
		while ticks - old == 0 {
			hint_core_should_pause();
			ticks = TIMER.get_clock_tick();
		}

		rmb();
		let start = unsafe { rdtsc() };
		/* wait 3 ticks to determine the frequency */
		while TIMER.get_clock_tick() - ticks < 3 {
				hint_core_should_pause();
		}
		rmb();
		let end = unsafe { rdtsc() };

		if end > start {
			diff = end - start;
		} else {
			diff = start - end;
		}

		let freq = (((TIMER_FREQ as u64) * diff) / (1000000u64 * 3u64)) as u32;

		freq
	};
}

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub fn mb()
{
	unsafe {
		asm!("mfence" ::: "memory" : "volatile");
	}
}

/// wait a few microseconds, realized by busy waiting
pub fn udelay(usecs: u64)
{
	unsafe {
		let end: u64 = rdtsc() + (get_cpu_frequency() as u64) * usecs;

		mb();
		while rdtsc() < end {
			mb();
			hint_core_should_pause();
		}
	}
}

/// Force strict CPU ordering, serializes load operations.
#[inline(always)]
pub fn rmb()
{
	unsafe {
		asm!("lfence" ::: "memory" : "volatile");
	}
}

/// Force strict CPU ordering, serializes store operations.
#[inline(always)]
pub fn wmb() {
	unsafe {
		asm!("sfence" ::: "memory" : "volatile");
	}
}

/// Search the first most significant bit
#[inline(always)]
pub fn msb(i: u64) -> u64 {
	let ret: u64;

	if i == 0 {
		ret = !0;
	} else {
		unsafe { asm!("bsr $1, $0" : "=r"(ret) : "r"(i) : "cc" : "volatile"); }
	}

	ret
}

/// Search the least significant bit
#[inline(always)]
pub fn lsb(i: u64) -> u64 {
	let ret: u64;

	if i == 0 {
		ret = !0;
	} else {
		unsafe { asm!("bsf $1, $0" : "=r"(ret) : "r"(i) : "cc" : "volatile"); }
	}

	ret
}

/// The halt function stops the processor until the next interrupt arrives
pub fn halt() {
	unsafe {
		asm!("hlt" :::: "volatile");
	}
}

/// returns the cpu frequency
pub fn get_cpu_frequency() -> u32 {
	*FREQUENCY
}

/// Shutdown the system, if the kernel is booted within Qemu
pub fn shutdown() -> ! {
	// shutdown, works only on Qemu
	unsafe {
		let mut shutdown_port : cpuio::Port<u8> = cpuio::Port::new(0xf4);
		shutdown_port.write(0x00);
	};

	loop {
		halt();
	}
}

/// Initialize processor dependent features
pub fn init() {
	debug!("enable supported processor features");

	let cpuid = CpuId::new();

	let mut cr0 = unsafe { control_regs::cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | control_regs::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | control_regs::CR0_NUMERIC_ERROR;
	cr0 = cr0 | control_regs::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(control_regs::CR0_CACHE_DISABLE|control_regs::CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { control_regs::cr0_write(cr0) };

	let mut cr4 = unsafe { control_regs::cr4() };

	let has_pge = match cpuid.get_feature_info() {
		Some(finfo) => finfo.has_pge(),
		None => false
	};

	if has_pge {
		cr4 |= CR4_ENABLE_GLOBAL_PAGES;
	}

	let has_fsgsbase = match cpuid.get_extended_feature_info() {
		Some(efinfo) => efinfo.has_fsgsbase(),
		None => false
	};

	if has_fsgsbase {
		cr4 |= CR4_ENABLE_FSGSBASE;
	}

	let has_mce = match cpuid.get_feature_info() {
		Some(finfo) => finfo.has_mce(),
		None => false
	};

	if has_mce {
		cr4 |= CR4_ENABLE_MACHINE_CHECK; // enable machine check exceptions
	}

	// disable performance monitoring counter
	// allow the usage of rdtsc in user space
	cr4 &= !(CR4_ENABLE_PPMC|CR4_TIME_STAMP_DISABLE);

	debug!("set CR4 to {:?}", cr4);

	unsafe { control_regs::cr4_write(cr4) };
}
