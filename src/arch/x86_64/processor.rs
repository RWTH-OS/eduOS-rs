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

use cpuio;
use x86::*;

// feature list 0x00000001 (ebx)
const CPU_FEATURE_FPU : u32 = (1 << 0);
const CPU_FEATURE_PSE : u32 = (1 << 3);
const CPU_FEATURE_MSR : u32 = (1 << 5);
const CPU_FEATURE_PAE : u32 = (1 << 6);
const CPU_FEATURE_APIC : u32 = (1 << 9);
const CPU_FEATURE_SEP : u32 = (1 << 11);
const CPU_FEATURE_PGE : u32 = (1 << 13);
const CPU_FEATURE_PAT : u32 = (1 << 16);
const CPU_FEATURE_PSE36 : u32 = (1 << 17);
const CPU_FEATURE_CLFLUSH : u32 = (1 << 19);
const CPU_FEATURE_MMX : u32 = (1 << 23);
const CPU_FEATURE_FXSR : u32 = (1 << 24);
const CPU_FEATURE_SSE : u32 = (1 << 25);
const CPU_FEATURE_SSE2 : u32 = (1 << 26);

// feature list 0x00000001 (ecx)
const CPU_FEATURE_MWAIT	: u32 = (1 << 3);
const CPU_FEATURE_VMX : u32 = (1 << 5);
const CPU_FEATURE_EST : u32 = (1 << 7);
const CPU_FEATURE_SSE3 : u32 = (1 << 9);
const CPU_FEATURE_FMA : u32 = (1 << 12);
const CPU_FEATURE_DCA : u32 = (1 << 18);
const CPU_FEATURE_SSE4_1 : u32 = (1 << 19);
const CPU_FEATURE_SSE4_2 : u32 = (1 << 20);
const CPU_FEATURE_X2APIC : u32 = (1 << 21);
const CPU_FEATURE_MOVBE	: u32 = (1 << 22);
const CPU_FEATURE_XSAVE	: u32 = (1 << 26);
const CPU_FEATURE_OSXSAVE : u32 = (1 << 27);
const CPU_FEATURE_AVX : u32 = (1 << 28);
const CPU_FEATURE_RDRAND : u32 = (1 << 30);
const CPU_FEATURE_HYPERVISOR : u32 = (1 << 31);

// const.80000001H:EDX feature list
const CPU_FEATURE_SYSCALL : u32 = (1 << 11);
const CPU_FEATURE_NX : u32 = (1 << 20);
const CPU_FEATURE_1GBHP : u32 = (1 << 26);
const CPU_FEATURE_RDTSCP : u32 = (1 << 27);
const CPU_FEATURE_LM : u32 = (1 << 29);

// feature list 0x00000007:0
const CPU_FEATURE_FSGSBASE : u32 = (1 << 0);
const CPU_FEATURE_TSC_ADJUST : u32 = (1 << 1);
const CPU_FEATURE_BMI1	: u32 = (1 << 3);
const CPU_FEATURE_HLE : u32	= (1 << 4);
const CPU_FEATURE_AVX2 : u32 = (1 << 5);
const CPU_FEATURE_SMEP : u32 = (1 << 7);
const CPU_FEATURE_BMI2 : u32 = (1 << 8);
const CPU_FEATURE_ERMS : u32 = (1 << 9);
const CPU_FEATURE_INVPCID : u32 = (1 << 10);
const CPU_FEATURE_RTM : u32 = (1 << 11);
const CPU_FEATURE_CQM : u32 = (1 << 12);
const CPU_FEATURE_MPX : u32 = (1 << 14);
const CPU_FEATURE_AVX512F : u32 = (1 << 16);
const CPU_FEATURE_RDSEED : u32 = (1 << 18);
const CPU_FEATURE_ADX : u32 = (1 << 19);
const CPU_FEATURE_SMAP : u32 = (1 << 20);
const CPU_FEATURE_PCOMMIT : u32 = (1 << 22);
const CPU_FEATURE_CLFLUSHOPT : u32 = (1 << 23);
const CPU_FEATURE_CLWB : u32 = (1 << 24);
const CPU_FEATURE_AVX512PF : u32 = (1 << 26);
const CPU_FEATURE_AVX512ER : u32 = (1 << 27);
const CPU_FEATURE_AVX512CD : u32 = (1 << 28);
const CPU_FEATURE_SHA_NI : u32 = (1 << 29);

// feature list 0x00000006
const CPU_FEATURE_IDA : u32 = (1 << 0);
const CPU_FEATURE_EPB : u32 = (1 << 3);
const CPU_FEATURE_HWP : u32 = (1 << 10);

/*
* EFLAGS bits
*/
const EFLAGS_CF : u32 = (1 <<  0); /* Carry Flag */
const EFLAGS_FIXED : u32 = (1 <<  1); /* Bit 1 - always on */
const EFLAGS_PF	: u32 = (1 <<  2); /* Parity Flag */
const EFLAGS_AF	: u32 = (1 <<  4); /* Auxiliary carry Flag */
const EFLAGS_ZF	: u32 = (1 <<  6); /* Zero Flag */
const EFLAGS_SF	: u32 = (1 <<  7); /* Sign Flag */
const EFLAGS_TF	: u32 = (1 <<  8); /* Trap Flag */
const EFLAGS_IF	: u32 = (1 <<  9); /* Interrupt Flag */
const EFLAGS_DF	: u32 = (1 << 10); /* Direction Flag */
const EFLAGS_OF	: u32 = (1 << 11); /* Overflow Flag */
const EFLAGS_IOPL : u32 = (1 << 12); /* I/O Privilege Level (2 bits) */
const EFLAGS_NT	: u32 = (1 << 14); /* Nested Task */
const EFLAGS_RF	: u32 = (1 << 16); /* Resume Flag */
const EFLAGS_VM	: u32 = (1 << 17); /* Virtual Mode */
const EFLAGS_AC	: u32 = (1 << 18); /* Alignment Check/Access Control */
const EFLAGS_VIF : u32 = (1 << 19); /* Virtual Interrupt Flag */
const EFLAGS_VIP : u32 = (1 << 20); /* Virtual Interrupt Pending */
const EFLAGS_ID : u32 = (1 << 21); /* const detection */

// x86 control registers

/// Protected Mode Enable
const CR0_PE : u64 = (1 << 0);
/// Monitor coprocessor
const CR0_MP : u64 = (1 << 1);
/// Enable FPU emulation
const CR0_EM : u64 = (1 << 2);
/// Task switched
const CR0_TS : u64 = (1 << 3);
/// Extension type of coprocessor
const CR0_ET : u64 = (1 << 4);
/// Enable FPU error reporting
const CR0_NE : u64 = (1 << 5);
/// Enable write protected pages
const CR0_WP : u64 = (1 << 16);
/// Enable alignment checks
const CR0_AM : u64 = (1 << 18);
/// Globally enables/disable write-back caching
const CR0_NW : u64 = (1 << 29);
/// Globally disable memory caching
const CR0_CD : u64 = (1 << 30);
/// Enable paging
const CR0_PG : u64 = (1 << 31);

/// Virtual 8086 Mode Extensions
const CR4_VME: u64 = (1 << 0);
/// Protected-mode Virtual Interrupts
const CR4_PVI : u64 = (1 << 1);
/// Disable Time Stamp Counter register (rdtsc instruction)
const CR4_TSD : u64 = (1 << 2);
/// Enable debug extensions
const CR4_DE : u64 = (1 << 3);
///  Enable hugepage support
const CR4_PSE : u64 = (1 << 4);
/// Enable physical address extension
const CR4_PAE : u64 = (1 << 5);
/// Enable machine check exceptions
const CR4_MCE : u64 = (1 << 6);
/// Enable global pages
const CR4_PGE : u64 = (1 << 7);
/// Enable Performance-Monitoring Counter
const CR4_PCE : u64 = (1 << 8);
/// Enable Operating system support for FXSAVE and FXRSTOR instructions
const CR4_OSFXSR : u64 = (1 << 9);
/// Enable Operating System Support for Unmasked SIMD Floating-Point Exceptions
const CR4_OSXMMEXCPT : u64 = (1 << 10);
/// Enable Virtual Machine Extensions, see Intel VT-x
const CR4_VMXE : u64 = (1 << 13);
/// Enable Safer Mode Extensions, see Trusted Execution Technology (TXT)
const CR4_SMXE : u64 = (1 << 14);
/// Enables the instructions RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE
const CR4_FSGSBASE : u64 = (1 << 16);
/// Enables process-context identifiers
const CR4_PCIDE	: u64 =	(1 << 17);
/// Enable XSAVE and Processor Extended States
const CR4_OSXSAVE : u64 = (1 << 18);
/// Enable Supervisor Mode Execution Protection
const CR4_SMEP : u64 = (1 << 20);
/// Enable Supervisor Mode Access Protection
const CR4_SMAP : u64 = (1 << 21);

// x86-64 specific MSRs

/// APIC register
const MSR_APIC_BASE : u32 = 0x0000001B;
/// extended feature register
const MSR_EFER : u32 = 0xc0000080;
/// legacy mode SYSCALL target
const MSR_STAR : u32 = 0xc0000081;
/// long mode SYSCALL target
const MSR_LSTAR : u32 = 0xc0000082;
/// compat mode SYSCALL target
const MSR_CSTAR : u32 = 0xc0000083;
/// EFLAGS mask for syscall
const MSR_SYSCALL_MASK : u32 = 0xc0000084;
/// 64bit FS base
const MSR_FS_BASE : u32 = 0xc0000100;
/// 64bit GS base
const MSR_GS_BASE : u32 = 0xc0000101;
/// SwapGS GS shadow
const MSR_KERNEL_GS_BASE : u32 = 0xc0000102;

const MSR_XAPIC_ENABLE : u32 = (1 << 11);
const MSR_X2APIC_ENABLE : u32 = (1 << 10);

const MSR_IA32_PLATFORM_ID : u32 = 0x00000017;

const MSR_IA32_PERFCTR0	: u32 = 0x000000c1;
const MSR_IA32_PERFCTR1	: u32 = 0x000000c2;
const MSR_FSB_FREQ : u32 = 0x000000cd;
const MSR_PLATFORM_INFO : u32 = 0x000000ce;

const MSR_IA32_MPERF : u32 = 0x000000e7;
const MSR_IA32_APERF : u32 = 0x000000e8;
const MSR_IA32_MISC_ENABLE : u32 = 0x000001a0;
const MSR_IA32_FEATURE_CONTROL : u32 = 0x0000003a;
const MSR_IA32_ENERGY_PERF_BIAS	: u32 = 0x000001b0;
const MSR_IA32_PERF_STATUS : u32 = 0x00000198;
const MSR_IA32_PERF_CTL : u32 = 0x00000199;
const MSR_IA32_CR_PAT : u32 = 0x00000277;
const MSR_MTRRDEFTYPE : u32 = 0x000002ff;

const MSR_PPERF : u32 = 0x0000064e;
const MSR_PERF_LIMIT_REASONS : u32 = 0x0000064f;
const MSR_PM_ENABLE : u32 = 0x00000770;
const MSR_HWP_CAPABILITIES : u32 = 0x00000771;
const MSR_HWP_REQUEST_PKG : u32 = 0x00000772;
const MSR_HWP_INTERRUPT : u32 = 0x00000773;
const MSR_HWP_REQUEST : u32 = 0x00000774;
const MSR_HWP_STATUS : u32 = 0x00000777;

const MSR_IA32_MISC_ENABLE_ENHANCED_SPEEDSTEP : u64 = (1 << 16);
const MSR_IA32_MISC_ENABLE_SPEEDSTEP_LOCK : u64 = (1 << 20);
const MSR_IA32_MISC_ENABLE_TURBO_DISABLE : u64 = (1 << 38);

const MSR_MTRRFIX64K_00000 : u32 = 0x00000250;
const MSR_MTRRFIX16K_80000 : u32 = 0x00000258;
const MSR_MTRRFIX16K_A0000 : u32 = 0x00000259;
const MSR_MTRRFIX4K_C0000 : u32 = 0x00000268;
const MSR_MTRRFIX4K_C8000 : u32 = 0x00000269;
const MSR_MTRRFIX4K_D0000 : u32 = 0x0000026a;
const MSR_MTRRFIX4K_D8000 : u32 = 0x0000026b;
const MSR_MTRRFIX4K_E0000 : u32 = 0x0000026c;
const MSR_MTRRFIX4K_E8000 : u32 = 0x0000026d;
const MSR_MTRRFIX4K_F0000 : u32 = 0x0000026e;
const MSR_MTRRFIX4K_F8000 : u32 = 0x0000026f;

const MSR_OFFCORE_RSP_0 : u32 = 0x000001a6;
const MSR_OFFCORE_RSP_1 : u32 = 0x000001a7;
const MSR_NHM_TURBO_RATIO_LIMIT : u32 = 0x000001ad;
const MSR_IVT_TURBO_RATIO_LIMIT : u32 = 0x000001ae;
const MSR_TURBO_RATIO_LIMIT : u32 = 0x000001ad;
const MSR_TURBO_RATIO_LIMIT1 : u32 = 0x000001ae;
const MSR_TURBO_RATIO_LIMIT2 : u32 = 0x000001af;

// MSR EFER bits
const EFER_SCE : u32 = (1 << 0);
const EFER_LME : u32 = (1 << 8);
const EFER_LMA : u32 = (1 << 10);
const EFER_NXE : u32 = (1 << 11);
const EFER_SVME : u32 = (1 << 12);
const EFER_LMSLE : u32 = (1 << 13);
const EFER_FFXSR : u32 = (1 << 14);
const EFER_TCE : u32 = (1 << 15);

pub fn halt() {
	loop {
		unsafe {
			asm!("hlt" :::: "volatile");
		}
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
	let mut cr0: u64 = unsafe { controlregs::cr0() };

	// be sure that AM, NE and MP is enabled
	cr0 = cr0 | CR0_AM;
	cr0 = cr0 | CR0_NE;
	cr0 = cr0 | CR0_MP;
	// enable cache
	cr0 = cr0 & !(CR0_CD|CR0_NW);

	unsafe { controlregs::cr0_write(cr0) };
}
