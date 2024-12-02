mod gdt;
pub mod irq;
mod pit;
pub(crate) mod processor;
#[cfg(not(feature = "vga"))]
pub(crate) mod serial;
#[cfg(target_arch = "x86_64")]
mod start;
pub(crate) mod switch;
mod syscall;
pub(crate) mod task;
#[cfg(feature = "vga")]
pub(crate) mod vga;

use crate::arch::x86::kernel::syscall::syscall_handler;
use crate::consts::USER_ENTRY;
use bootloader::BootInfo;
use core::arch::asm;

#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("entry32.s"));

#[cfg(target_arch = "x86_64")]
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub fn register_task() {
	let sel: u16 = 6u16 << 3;

	unsafe {
		asm!("ltr ax", in("ax") sel, options(nostack, nomem));
	}
}

pub unsafe fn jump_to_user_land(func: extern "C" fn()) -> ! {
	let ds = 0x23u64;
	let cs = 0x2bu64;
	let addr: usize = USER_ENTRY.as_usize() | (func as usize & 0xFFFusize);

	asm!(
		"swapgs",
		"push {0}",
		"push {1}",
		"pushf",
		"push {2}",
		"push {3}",
		"iretq",
		in(reg) ds,
		in(reg) USER_ENTRY.as_u64() + 0x400000u64,
		in(reg) cs,
		in(reg) addr,
		options(nostack)
	);

	loop {
		processor::halt();
	}
}

/// This macro can be used to call system functions from user-space
#[macro_export]
macro_rules! syscall {
	($arg0:expr) => {
		arch::x86::kernel::syscall0($arg0 as u64)
	};

	($arg0:expr, $arg1:expr) => {
		arch::x86::kernel::syscall1($arg0 as u64, $arg1 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr) => {
		arch::x86::kernel::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
		arch::x86::kernel::syscall3($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
		arch::x86::kernel::syscall4(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
		arch::x86::kernel::syscall5(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {
		arch::x86::kernel::syscall6(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
			$arg6 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr, $arg7:expr) => {
		arch::x86::kernel::syscall7(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
			$arg6 as u64,
			$arg7 as u64,
		)
	};
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall0(arg0: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall1(arg0: u64, arg1: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			in("rsi") arg2,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			in("rsi") arg2,
			in("rdx") arg3,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall4(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			in("rsi") arg2,
			in("rdx") arg3,
			in("r10") arg4,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall5(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			in("rsi") arg2,
			in("rdx") arg3,
			in("r10") arg4,
			in("r8") arg5,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall6(
	arg0: u64,
	arg1: u64,
	arg2: u64,
	arg3: u64,
	arg4: u64,
	arg5: u64,
	arg6: u64,
) -> u64 {
	let mut ret: u64;
	unsafe {
		asm!("syscall",
			inlateout("rax") arg0 => ret,
			in("rdi") arg1,
			in("rsi") arg2,
			in("rdx") arg3,
			in("r10") arg4,
			in("r8") arg5,
			in("r9") arg6,
			lateout("rcx") _,
			lateout("r11") _,
			options(preserves_flags, nostack)
		);
	}
	ret
}

/// Initialize module, must be called once, and only once
pub(crate) fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();

	#[cfg(feature = "vga")]
	vga::init();
}
