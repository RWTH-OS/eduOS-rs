pub mod serial;
pub mod processor;
pub mod task;
pub mod irq;
mod gdt;
mod pit;
mod start;
mod switch;
mod syscall;

pub use arch::x86_64::syscall::syscall_handler;

pub fn register_task()
{
	let sel: u16 = 6u16 << 3;

	unsafe { asm!("ltr $0" :: "r"(sel) :: "volatile"); }
}

#[naked]
pub fn jump_to_user_land(func: fn() -> !) -> !
{
	let ds = 0x23u64;
	let cs = 0x2bu64;

	unsafe {
		asm!("mov $0, %ds; mov $0, %es; push $0; push %rsp; addq $$16, (%rsp); pushfq; push $1; push $2; iretq"
			:: "r"(ds), "r"(cs), "r"(func as u64)
			:: "volatile");
	}

	loop {
		processor::halt();
	}
}

#[macro_export]
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

/// Initialize module, must be called once, and only once
pub fn init() {
	processor::init();
	gdt::init();
	irq::init();
	pit::init();
}
