#![allow(dead_code)]

use cpuio;
use raw_cpuid::*;
use logging::*;
use x86::shared::*;
use x86::shared::control_regs::*;
use x86::shared::msr::*;
use arch::x86_64::syscall_handler;

// MSR EFER bits
const EFER_SCE: u64 = (1 << 0);
const EFER_LME: u64 = (1 << 8);
const EFER_LMA: u64 = (1 << 10);
const EFER_NXE: u64 = (1 << 11);
const EFER_SVME: u64 = (1 << 12);
const EFER_LMSLE: u64 = (1 << 13);
const EFER_FFXSR: u64 = (1 << 14);
const EFER_TCE: u64 = (1 << 15);

/// Force strict CPU ordering, serializes load and store operations.
#[inline(always)]
pub fn mb()
{
	unsafe {
		asm!("mfence" ::: "memory" : "volatile");
	}
}

/// Search the most significant bit
#[inline(always)]
pub fn msb(value: u64) -> Option<u64> {
	if value > 0 {
		let ret: u64;
		unsafe { asm!("bsr $1, $0" : "=r"(ret) : "r"(value) : "cc" : "volatile"); }
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
		unsafe { asm!("bsf $1, $0" : "=r"(ret) : "r"(value) : "cc" : "volatile"); }
		Some(ret)
	} else {
		None
	}
}

pub fn halt() {
	unsafe {
		asm!("hlt" :::: "volatile");
	}
}

pub fn pause() {
	unsafe {
		asm!("pause" :::: "volatile");
	}
}

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
	// shutdown, works like Qemu's shutdown command
	unsafe {
		let mut shutdown_port : cpuio::Port<u8> = cpuio::Port::new(0xf4);
		shutdown_port.write(0x00);
	};

	loop {
		halt();
	}
}

pub fn init() {
	debug!("enable supported processor features");

	let cpuid = CpuId::new();

	let mut cr0 = unsafe { cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | CR0_ALIGNMENT_MASK;
	cr0 = cr0 | CR0_NUMERIC_ERROR;
	cr0 = cr0 | CR0_MONITOR_COPROCESSOR;
	// enable cache
	cr0 = cr0 & !(CR0_CACHE_DISABLE|CR0_NOT_WRITE_THROUGH);

	debug!("set CR0 to {:?}", cr0);

	unsafe { cr0_write(cr0) };

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

	let has_syscall = match cpuid.get_extended_function_info() {
		Some(finfo) => finfo.has_syscall_sysret(),
		None => false
	};

	if has_syscall == false {
		panic!("Syscall support is missing");
	}

	// enable support of syscall and sysret
	unsafe {
		wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_LMA | EFER_SCE);
		wrmsr(IA32_STAR, (0x1Bu64 << 48) | (0x08u64 << 32));
		wrmsr(IA32_LSTAR, syscall_handler as u64);
		wrmsr(IA32_FMASK, 1 << 9); // clear IF flag during system call
	}

	/*print!("Detected processor: ");
	match cpuid.get_extended_function_info() {
		Some(exinfo) => {
			match exinfo.processor_brand_string() {
				Some(str) => println!("{}", str),
				None => println!("unknwon")
			}
		},
		None => println!("unknwon")
	}

	println!("Summary of cache information:");
	match cpuid.get_cache_parameters() {
		Some(cparams) => {
			for cache in cparams {
				let size = cache.associativity() * cache.physical_line_partitions() * cache.coherency_line_size() * cache.sets();
				println!("L{}-Cache size is {}", cache.level(), size);
			}
		},
		None => println!("No cache parameter information available"),
	}
	println!("");*/
}
