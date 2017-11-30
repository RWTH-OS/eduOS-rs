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

pub mod serial;
pub mod processor;
pub mod task;
pub mod irq;
pub mod pit;
pub mod gdt;

/// Invokes an OS system-call handler at privilege level 0.
///
/// It does so by loading RIP from the IA32_LSTAR MSR (after saving the address of the instruction following SYSCALL into RCX).
///
/// "A.2 AMD64 Linux Kernel Conventions" of System V Application Binary Interface AMD64 Architecture Processor Supplement:
///
/// * The kernel interface uses %rdi, %rsi, %rdx, %r10, %r8 and %r9.
/// * A system-call is done via the syscall instruction. The kernel destroys registers %rcx and %r11.
/// * The number of the syscall has to be passed in register %rax.
/// * System-calls are limited to six arguments, no argument is passed directly on the stack.
/// * Returning from the syscall, register %rax contains the result of the system-call. A value in the range between -4095 and -1 indicates an error, it is -errno.
/// * Only values of class INTEGER or class MEMORY are passed to the kernel.
///
/// This code is derived by the https://github.com/gz/rust-x86/blob/master/src/bits64/syscall.rs.

macro_rules! syscall {
    ($arg0:expr)
        => ( arch::x86_64::syscall0($arg0 as u64) );

    ($arg0:expr, $arg1:expr)
        => ( arch::x86_64::syscall1($arg0 as u64, $arg1 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr)
        => ( arch::x86_64::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr)
        => ( arch::x86_64::syscall3($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr)
        => ( arch::x86_64::syscall4($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr)
        => ( arch::x86_64::syscall5($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr)
        => ( arch::x86_64::syscall6($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64, $arg6 as u64) );

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr, $arg7:expr)
        => ( arch::x86_64::syscall7($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64, $arg4 as u64, $arg5 as u64, $arg6 as u64, $arg7 as u64) );
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall0(arg0: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret) : "{rax}" (arg0) : "rcx", "r11", "memory" : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall1(arg0: u64, arg1: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1)
                   : "rcx", "r11", "memory" : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2)
                   : "rcx", "r11", "memory" : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret) : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2), "{rdx}" (arg3)
                   : "rcx", "r11", "memory" : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall4(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret)
                   : "{rax}"  (arg0), "{rdi}"  (arg1), "{rsi}"  (arg2), "{rdx}"  (arg3), "{r10}"  (arg4)
                   : "rcx", "r11", "memory" : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall5(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret)
                   : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2), "{rdx}" (arg3), "{r10}" (arg4), "{r8}" (arg5)
                   : "rcx", "r11", "memory"
                   : "volatile");
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub unsafe fn syscall6(arg0: u64,
                       arg1: u64,
                       arg2: u64,
                       arg3: u64,
                       arg4: u64,
                       arg5: u64,
                       arg6: u64)
                       -> u64 {
    let mut ret: u64;
    asm!("syscall" : "={rax}" (ret)
                   : "{rax}" (arg0), "{rdi}" (arg1), "{rsi}" (arg2), "{rdx}" (arg3),
                     "{r10}" (arg4), "{r8}" (arg5), "{r9}" (arg6)
                   : "rcx", "r11", "memory"
                   : "volatile");
    ret
}
