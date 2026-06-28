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
/// This installs the trap vector and starts the periodic CLINT timer that
/// drives the preemptive scheduler. Interrupts stay globally masked
/// (`mstatus.MIE` is cleared at reset) until the kernel explicitly enables them
/// via `irq::irq_enable`.
pub(crate) fn init() {
	setup_pmp();
	irq::init();
	timer::init();
}

/// Allow U-mode to access the whole address space.
///
/// Without a Physical Memory Protection entry, U-mode (and S-mode) accesses are
/// denied by default, so a user task could not even fetch its first instruction.
/// As there is no paging yet, we grant U-mode read/write/execute access to the
/// entire address space via a single Top-Of-Range PMP entry.
fn setup_pmp() {
	unsafe {
		asm!(
			"csrw pmpaddr0, {addr}",
			"csrw pmpcfg0, {cfg}",
			addr = in(reg) usize::MAX,
			cfg = in(reg) 0x0f_usize, // A = TOR, R | W | X
			options(nostack, nomem),
		);
	}
}

/// Architectures with a hardware task register load it here. RISC-V dispatches
/// traps through `mtvec` and needs no such registration.
pub fn register_task() {}

/// Helper function to jump into the user space (privilege mode U).
///
/// The user task keeps using the current (main) stack and runs in U-mode; on a
/// trap (system call via `ecall` or a timer interrupt) the machine-mode handler
/// reuses that same stack, which is fine because there is no paging yet and
/// U-mode shares the kernel's flat address space, separated only by its
/// privilege level.
///
/// # Safety
///
/// `func` must be a valid entry point for the user task.
pub unsafe fn jump_to_user_land(func: extern "C" fn()) -> ! {
	// mstatus.MPP (bits 12:11) selects the privilege mode `mret` returns to;
	// clearing it selects U-mode. mstatus.MPIE (bit 7) is restored into MIE.
	unsafe {
		asm!(
			"csrc mstatus, {mpp}",   // MPP = 0b00 => U-mode
			"csrs mstatus, {mpie}",  // MPIE = 1
			"csrw mepc, {entry}",
			"mret",
			mpp = in(reg) 0b11usize << 11,
			mpie = in(reg) 1usize << 7,
			entry = in(reg) func as usize,
			options(noreturn),
		);
	}
}

/// System call wrappers. The system-call number is passed in `a7`, the
/// arguments in `a0`-`a5`, and the result is returned in `a0`. The kernel side
/// is implemented in `irq::handle_trap` (the `ecall` exception from U-mode).
#[inline(always)]
pub fn syscall0(arg0: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("ecall", in("a7") arg0, lateout("a0") ret, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall1(arg0: u64, arg1: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("ecall", in("a7") arg0, inlateout("a0") arg1 => ret, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!("ecall", in("a7") arg0, inlateout("a0") arg1 => ret, in("a1") arg2, options(nostack));
	}
	ret
}

#[inline(always)]
pub fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
	let ret: u64;
	unsafe {
		asm!(
			"ecall",
			in("a7") arg0,
			inlateout("a0") arg1 => ret,
			in("a1") arg2,
			in("a2") arg3,
			options(nostack),
		);
	}
	ret
}

#[macro_export]
macro_rules! syscall {
	($arg0:expr) => {
		$crate::arch::riscv64::kernel::syscall0($arg0 as u64)
	};

	($arg0:expr, $arg1:expr) => {
		$crate::arch::riscv64::kernel::syscall1($arg0 as u64, $arg1 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr) => {
		$crate::arch::riscv64::kernel::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
		$crate::arch::riscv64::kernel::syscall3(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
		)
	};
}
