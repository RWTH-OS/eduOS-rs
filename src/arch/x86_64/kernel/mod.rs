// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod serial;
pub mod processor;
pub mod task;
pub mod irq;
pub mod switch;
mod gdt;
mod pit;
mod start;
mod syscall;

use core::ptr::read_volatile;
use consts::*;
use logging::*;
pub use arch::x86_64::kernel::syscall::syscall_handler;

#[repr(C)]
struct KernelHeader {
	magic_number: u32,
	version: u32,
	mem_limit: u64,
	num_cpus: u32,
	file_addr: u64,
	file_length: u64
}

/// Kernel header to announce machine features
#[link_section = ".kheader"]
static KERNEL_HEADER: KernelHeader = KernelHeader {
	magic_number: 0xDEADC0DEu32,
	version: 0,
	mem_limit: 0,
	num_cpus: 1,
	file_addr: 0,
	file_length: 0
};

pub fn get_memfile() -> (u64, u64) {
	let version = unsafe { read_volatile(&KERNEL_HEADER.version) };
	let addr = unsafe { read_volatile(&KERNEL_HEADER.file_addr) };
	let len = unsafe { read_volatile(&KERNEL_HEADER.file_length) };

	if version > 0 {
		(addr, len)
	} else {
		(0, 0)
	}
}

pub fn get_memory_size() -> usize {
		unsafe { read_volatile(&KERNEL_HEADER.mem_limit) as usize }
}

pub fn register_task() {
	let sel: u16 = 6u16 << 3;

	unsafe { asm!("ltr $0" :: "r"(sel) :: "volatile"); }
}

#[inline(never)]
#[naked]
pub fn jump_to_user_land(entry: u64) -> !
{
	let ds = 0x23u64;
	let cs = 0x2bu64;
	let stack = USER_STACK - 0x100;

	debug!("Set user space stack to 0x{:x}", stack);

	unsafe {
		asm!("swapgs; mov $0, %ds; mov $0, %es; push $0; push $3; pushfq; push $1; push $2; iretq"
			:: "r"(ds), "r"(cs), "r"(entry as u64), "r"(stack)
			:: "volatile");
	}

	loop {
		processor::halt();
	}
}

/// This macro can be used to call system functions from user-space
#[macro_export]
macro_rules! syscall {
	($arg0:expr)
		=> ( arch::x86_64::kernel::syscall0($arg0 as u64) );

	($arg0:expr, $arg1:expr)
		=> ( arch::x86_64::kernel::syscall1($arg0 as u64, $arg1 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr)
		=> ( arch::x86_64::kernel::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr)
		=> ( arch::x86_64::kernel::syscall3($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr)
		=> ( arch::x86_64::kernel::syscall4($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr)
		=> ( arch::x86_64::kernel::syscall5($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr)
		=> ( arch::x86_64::kernel::syscall6($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64, $arg6 as u64) );

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr, $arg7:expr)
		=> ( arch::x86_64::kernel::syscall7($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64, $arg6 as u64, $arg7 as u64) );
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall0(arg0: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall" : "={rax}" (ret) : "{rax}" (arg0) : "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall1(arg0: u64, arg1: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1)
						: "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2)
						: "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2),
						"{rdx}" (arg3) : "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall4(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret)
						: "{rax}"  (arg0), "{rdi}"  (arg1), "{rsi}"  (arg2), "{rdx}"  (arg3),
						"{r10}"  (arg4) : "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall5(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret)
						: "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2), "{rdx}" (arg3),
						"{r10}" (arg4), "{r8}" (arg5) : "rcx", "r11", "memory" : "volatile");
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall6(arg0: u64,
	arg1: u64,
	arg2: u64,
	arg3: u64,
	arg4: u64,
	arg5: u64,
	arg6: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall"	: "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2),
						"{rdx}" (arg3), "{r10}" (arg4), "{r8}" (arg5), "{r9}" (arg6)
						: "rcx", "r11", "memory" : "volatile");
	}
	ret
}

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();
}
