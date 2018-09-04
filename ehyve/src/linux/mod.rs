pub mod error;
pub mod ehyve;
pub mod vcpu;
pub mod utils;
pub mod kvm;

use std;
use libc::{c_int, size_t, c_void};

static mut KVMFD: c_int = -1;

#[allow(missing_copy_implementations)]
#[repr(C)]
pub struct Cpuid2 {
    pub nent: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Copy)]
pub struct Dtable {
    pub base: u64,
    pub limit: u16,
    pub padding: [u16; 3usize],
}

impl std::clone::Clone for Dtable {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Dtable {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy)]
pub struct Segment {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub type_: u8,
    pub present: u8,
    pub dpl: u8,
    pub db: u8,
    pub s: u8,
    pub l: u8,
    pub g: u8,
    pub avl: u8,
    pub unusable: u8,
    pub padding: u8,
}

impl std::clone::Clone for Segment {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Segment {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy)]
pub struct Sregs {
    pub cs: Segment,
    pub ds: Segment,
    pub es: Segment,
    pub fs: Segment,
    pub gs: Segment,
    pub ss: Segment,
    pub tr: Segment,
    pub ldt: Segment,
    pub gdt: Dtable,
    pub idt: Dtable,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4usize],
}

impl std::clone::Clone for Sregs {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Copy)]
pub struct Regs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl std::clone::Clone for Regs {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Regs {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl std::default::Default for Sregs {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

extern {
	fn kvm_init() -> c_int;
    fn kvm_create_vm(fd: c_int, flags: c_int) -> c_int;
	fn kvm_init_vm(fd: c_int, guest_size: size_t) -> *mut c_void;
    fn kvm_create_vcpu(fd: c_int, vcpu_id: c_int) -> c_int;
	fn kvm_init_vcpu(vcpufd: c_int, cpuid: c_int, elf_entry: size_t) ->  c_int;
	fn kvm_map_run(fd: c_int, cpufd: c_int) -> *mut c_void;
    fn kvm_run(fd: c_int) -> c_int;
    fn kvm_get_regs(fd: c_int, regs: *mut Regs) -> c_int;
    fn kvm_set_regs(fd: c_int, regs: *const Regs) -> c_int;
    fn kvm_get_sregs(fd: c_int, sregs: *mut Sregs) -> c_int;
    fn kvm_set_sregs(fd: c_int, sregs: *const Sregs) -> c_int;
}
