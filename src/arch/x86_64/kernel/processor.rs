#![allow(dead_code)]

use crate::arch::mm::get_boot_stack;
use crate::arch::x86_64::kernel::syscall_handler;
use crate::logging::*;
use crate::scheduler::task::Stack;
use core::arch::asm;
use qemu_exit::QEMUExit;
use x86::controlregs::*;
use x86::cpuid::*;
use x86::msr::*;

// MSR EFER bits
const EFER_SCE: u64 = 1 << 0;
const EFER_LME: u64 = 1 << 8;
const EFER_LMA: u64 = 1 << 10;
const EFER_NXE: u64 = 1 << 11;
const EFER_SVME: u64 = 1 << 12;
const EFER_LMSLE: u64 = 1 << 13;
const EFER_FFXSR: u64 = 1 << 14;
const EFER_TCE: u64 = 1 << 15;

static mut PHYSICAL_ADDRESS_BITS: u8 = 0;
static mut LINEAR_ADDRESS_BITS: u8 = 0;
static mut SUPPORTS_1GIB_PAGES: bool = false;

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub fn mb() {
	unsafe {
		asm!("mfence", options(preserves_flags, nostack));
	}
}

/// Search the most significant bit
#[inline(always)]
pub fn msb(value: u64) -> Option<u64> {
	if value > 0 {
		let ret: u64;
		unsafe {
			asm!("bsr {0}, {1}",
				out(reg) ret,
				in(reg) value,
				options(nomem, nostack)
			);
		}
		Some(ret)
	} else {
		None
	}
}

/// Search the least significant bit
#[inline(always)]
pub fn lsb(value: u64) -> Option<u64> {
	if value > 0 {
		let ret: u64;
		unsafe {
			asm!("bsf {0}, {1}",
				out(reg) ret,
				in(reg) value,
				options(nomem, nostack)
			);
		}
		Some(ret)
	} else {
		None
	}
}

#[inline(always)]
pub fn halt() {
	unsafe {
		asm!("hlt", options(nomem, nostack));
	}
}

#[inline(always)]
pub fn pause() {
	unsafe {
		asm!("pause", options(nomem, nostack));
	}
}

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	// shutdown, works like Qemu's shutdown command
	let qemu_exit_handle = qemu_exit::X86::new(0xf4, 5);
	qemu_exit_handle.exit_success();
}

pub fn supports_1gib_pages() -> bool {
	unsafe { SUPPORTS_1GIB_PAGES }
}

pub fn get_linear_address_bits() -> u8 {
	unsafe { LINEAR_ADDRESS_BITS }
}

pub fn get_physical_address_bits() -> u8 {
	unsafe { PHYSICAL_ADDRESS_BITS }
}

pub fn init() {
	debug!("enable supported processor features");

	let cpuid = CpuId::new();

	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | Cr0::CR0_ALIGNMENT_MASK;
	cr0 = cr0 | Cr0::CR0_NUMERIC_ERROR;
	cr0 = cr0 | Cr0::CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { cr0_write(cr0) };

	let mut cr4 = unsafe { cr4() };

	let has_pge = match cpuid.get_feature_info() {
		Some(finfo) => finfo.has_pge(),
		None => false,
	};

	if has_pge {
		cr4 |= Cr4::CR4_ENABLE_GLOBAL_PAGES;
	}

	let has_fsgsbase = match cpuid.get_extended_feature_info() {
		Some(efinfo) => efinfo.has_fsgsbase(),
		None => false,
	};

	if has_fsgsbase {
		cr4 |= Cr4::CR4_ENABLE_FSGSBASE;
	} else {
		panic!("eduOS-rs requires the CPU feature FSGSBASE");
	}

	let has_mce = match cpuid.get_feature_info() {
		Some(finfo) => finfo.has_mce(),
		None => false,
	};

	if has_mce {
		cr4 |= Cr4::CR4_ENABLE_MACHINE_CHECK; // enable machine check exceptions
	}

	// disable performance monitoring counter
	// allow the usage of rdtsc in user space
	cr4 &= !(Cr4::CR4_ENABLE_PPMC | Cr4::CR4_TIME_STAMP_DISABLE);

	debug!("set CR4 to {:?}", cr4);

	unsafe { cr4_write(cr4) };

	let has_syscall = match cpuid.get_extended_processor_and_feature_identifiers() {
		Some(finfo) => finfo.has_syscall_sysret(),
		None => false,
	};

	if has_syscall == false {
		panic!("Syscall support is missing");
	}

	// enable support of syscall and sysret
	unsafe {
		wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_LMA | EFER_SCE | EFER_NXE);
		wrmsr(IA32_STAR, (0x1Bu64 << 48) | (0x08u64 << 32));
		wrmsr(IA32_LSTAR, syscall_handler as u64);
		wrmsr(IA32_FMASK, 1 << 9); // clear IF flag during system call

		// reset GS registers
		wrmsr(IA32_GS_BASE, 0);
		asm!("wrgsbase {}", in(reg) get_boot_stack().top(), options(preserves_flags, nomem, nostack));
	}

	// determin processor features
	let extended_feature_info = cpuid
		.get_processor_capacity_feature_info()
		.expect("CPUID Capacity Feature Info is not available!");
	unsafe {
		PHYSICAL_ADDRESS_BITS = extended_feature_info.physical_address_bits();
		LINEAR_ADDRESS_BITS = extended_feature_info.linear_address_bits();
		SUPPORTS_1GIB_PAGES = cpuid
			.get_extended_processor_and_feature_identifiers()
			.expect("CPUID Extended Processor and Feature Info is not available!")
			.has_1gib_pages();
	}

	if supports_1gib_pages() {
		info!("System supports 1GiB pages");
	}
	debug!("Physical address bits {}", get_physical_address_bits());
	debug!("Linear address bits {}", get_linear_address_bits());
	debug!("CR0: {:?}", cr0);
	debug!("CR4: {:?}", cr4);
}
