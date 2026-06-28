pub mod gic;
pub mod irq;
pub(crate) mod processor;
pub(crate) mod serial;
pub(crate) mod start;
pub(crate) mod switch;
pub(crate) mod task;
pub mod timer;

use core::arch::asm;

/// Initialize the architecture specific interrupt infrastructure.
///
/// This installs the exception vector table, brings up the GICv3 interrupt
/// controller and starts the periodic timer that drives the preemptive
/// scheduler. Interrupts stay masked (see `start::pre_init`) until the kernel
/// explicitly enables them via `irq::irq_enable`.
pub(crate) fn init() {
	irq::init();
	gic::init();
	timer::init();
}

/// Architectures with a hardware task register load it here. aarch64 dispatches
/// exceptions through the vector table and needs no such registration.
pub fn register_task() {}

/// Helper function to jump into the user space (EL0).
///
/// The user task runs on the current (main) stack via `SP_EL0`, while `SP_EL1`
/// is switched to the per-task interrupt stack so that the `svc` system-call
/// trap and any exception from EL0 land on a separate kernel stack. There is no
/// paging yet, so EL0 shares the kernel's flat address space and is only
/// separated by its privilege level.
///
/// Interrupts are masked while in EL0, i.e. the user task runs until it issues a
/// system call (see `irq::handle_exception`).
///
/// # Safety
///
/// `func` must be a valid entry point for the user task.
pub unsafe fn jump_to_user_land(func: extern "C" fn()) -> ! {
	let kernel_sp = crate::scheduler::get_current_interrupt_stack().as_usize() as u64;
	// SPSR_EL1: mode EL0t (0b0000) with IRQ (I) and FIQ (F) masked.
	let spsr: u64 = 0b1100_0000;

	// Keep using the current (main) stack as the EL0 stack.
	let user_sp: u64;
	unsafe {
		asm!("mov {}, sp", out(reg) user_sp, options(nomem, nostack));
	}

	unsafe {
		asm!(
			"msr sp_el0, {usp}",   // EL0 stack
			"msr spsr_el1, {spsr}",
			"msr elr_el1, {entry}",
			"mov sp, {ksp}",       // SP_EL1 = interrupt stack for traps from EL0
			"eret",
			usp = in(reg) user_sp,
			spsr = in(reg) spsr,
			entry = in(reg) func as usize,
			ksp = in(reg) kernel_sp,
			options(noreturn),
		);
	}
}

/// System call wrappers. The system-call number is passed in `x8`, the
/// arguments in `x0`-`x7`, and the result is returned in `x0`. The kernel side
/// is implemented in `irq::handle_exception` (the `svc` synchronous exception).
#[inline(always)]
pub fn syscall0(arg0: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("svc #0", in("x8") arg0, lateout("x0") ret, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall1(arg0: u64, arg1: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("svc #0", in("x8") arg0, inlateout("x0") arg1 => ret, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("svc #0", in("x8") arg0, inlateout("x0") arg1 => ret, in("x1") arg2, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!(
			"svc #0",
			in("x8") arg0,
			inlateout("x0") arg1 => ret,
			in("x1") arg2,
			in("x2") arg3,
			options(nostack),
		);
	}
	ret
}

#[macro_export]
macro_rules! syscall {
	($arg0:expr) => {
		$crate::arch::aarch64::kernel::syscall0($arg0 as u64)
	};

	($arg0:expr, $arg1:expr) => {
		$crate::arch::aarch64::kernel::syscall1($arg0 as u64, $arg1 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr) => {
		$crate::arch::aarch64::kernel::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
		$crate::arch::aarch64::kernel::syscall3(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
		)
	};
}
