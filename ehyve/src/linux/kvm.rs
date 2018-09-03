use std;
use std::fmt;

/// KVM `run` exit reasons
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u32)]
pub enum Exit {
    Unknown = 0,
    Exception,
    Io,
    Hypercall,
    Debug,
    Hlt,
    Mmio,
    IrqWindowOpen,
    Shutdown,
    FailEntry,
    Intr,
    SetTpr,
    TprAccess,
    S390Sieic,
    S390Reset,
    Dcr,
    Nmi,
    InternalError,
    Osi,
    PaprHcall,
    S390Ucontrol,
    Watchdog,
    S390Tsch,
    Epr,
    SystemEvent,
}

#[repr(C)]
#[derive(Copy)]
pub struct SyncRegs;

impl ::std::clone::Clone for SyncRegs {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::std::default::Default for SyncRegs {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed9 {
    pub hardware_exit_reason: u64,
}

impl std::clone::Clone for Struct_Unnamed9 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed9 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed10 {
    pub hardware_entry_failure_reason: u64,
}

impl ::std::clone::Clone for Struct_Unnamed10 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed10 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed11 {
    pub exception: u32,
    pub error_code: u32,
}

impl std::clone::Clone for Struct_Unnamed11 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed11 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IoDirection {
    In,
    Out,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct ExitIo {
    pub direction: IoDirection,
    pub size: u8,
    pub port: u16,
    pub count: u32,
    pub data_offset: u64,
}

impl std::clone::Clone for ExitIo {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for ExitIo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Debug)]
pub struct DebugExitArch {
    pub exception: u32,
    pub pad: u32,
    pub pc: u64,
    pub dr6: u64,
    pub dr7: u64,
}

impl std::clone::Clone for DebugExitArch {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::std::default::Default for DebugExitArch {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed13 {
    pub arch: DebugExitArch,
}

impl std::clone::Clone for Struct_Unnamed13 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed13 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed14 {
    pub phys_addr: u64,
    pub data: [u8; 8usize],
    pub len: u32,
    pub is_write: u8,
}

impl std::clone::Clone for Struct_Unnamed14 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed14 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed15 {
    pub nr: u64,
    pub args: [u64; 6usize],
    pub ret: u64,
    pub longmode: u32,
    pub pad: u32,
}

impl std::clone::Clone for Struct_Unnamed15 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed15 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed16 {
    pub rip: u64,
    pub is_write: u32,
    pub pad: u32,
}

impl ::std::clone::Clone for Struct_Unnamed16 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed16 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed17 {
    pub icptcode: u8,
    pub ipa: u16,
    pub ipb: u32,
}

impl std::clone::Clone for Struct_Unnamed17 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed17 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed18 {
    pub trans_exc_code: u64,
    pub pgm_code: u32,
}

impl std::clone::Clone for Struct_Unnamed18 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed18 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed19 {
    pub dcrn: u32,
    pub data: u32,
    pub is_write: u8,
}

impl std::clone::Clone for Struct_Unnamed19 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed19 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed20 {
    pub suberror: u32,
    pub ndata: u32,
    pub data: [u64; 16usize],
}

impl std::clone::Clone for Struct_Unnamed20 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed20 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed21 {
    pub gprs: [u64; 32usize],
}
impl std::clone::Clone for Struct_Unnamed21 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed21 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed22 {
    pub nr: u64,
    pub ret: u64,
    pub args: [u64; 9usize],
}

impl std::clone::Clone for Struct_Unnamed22 {
    fn clone(&self) -> Self {
        *self
    }
}
impl std::default::Default for Struct_Unnamed22 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed23 {
    pub subchannel_id: u16,
    pub subchannel_nr: u16,
    pub io_int_parm: u32,
    pub io_int_word: u32,
    pub ipb: u32,
    pub dequeued: u8,
}

impl std::clone::Clone for Struct_Unnamed23 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed23 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed24 {
    pub epr: u32,
}

impl ::std::clone::Clone for Struct_Unnamed24 {
    fn clone(&self) -> Self {
        *self
    }
}
impl std::default::Default for Struct_Unnamed24 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_Unnamed25 {
    pub _type: u32,
    pub flags: u64,
}

impl std::clone::Clone for Struct_Unnamed25 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Struct_Unnamed25 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[allow(missing_docs, missing_debug_implementations)]
#[repr(C)]
#[derive(Copy)]
pub struct Union_Unnamed26 {
    pub _bindgen_data_: [u8; 1024usize],
}

#[allow(missing_docs)]
impl Union_Unnamed26 {
    pub fn regs(&self) -> *const SyncRegs {
        unsafe {
            let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
            ::std::mem::transmute(raw.offset(0))
        }
    }
    pub fn regs_mut(&mut self) -> *mut SyncRegs {
        unsafe {
            let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
            ::std::mem::transmute(raw.offset(0))
        }
    }
}

impl std::clone::Clone for Union_Unnamed26 {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Union_Unnamed26 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

/// Information about the reason `run` returned
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy)]
pub struct Run {
    request_interrupt_window: u8,
    padding1: [u8; 7usize],
    pub exit_reason: Exit,
    pub ready_for_interrupt_injection: u8,
    pub if_flag: u8,
    pub flags: u16,
    pub cr8: u64,
    pub apic_base: u64,
    _bindgen_data_1_: [u64; 32usize],
    pub kvm_valid_regs: u64,
    pub kvm_dirty_regs: u64,
    pub s: Union_Unnamed26,
}

#[allow(missing_docs)]
impl Run {
    pub fn hw(&self) -> *const Struct_Unnamed9 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn hw_mut(&mut self) -> *mut Struct_Unnamed9 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn fail_entry(&self) -> *const Struct_Unnamed10 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn fail_entry_mut(&mut self) -> *mut Struct_Unnamed10 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn ex(&self) -> *const Struct_Unnamed11 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn ex_mut(&mut self) -> *mut Struct_Unnamed11 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn io(&self) -> *const ExitIo {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn io_mut(&mut self) -> *mut ExitIo {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
        	std::mem::transmute(raw.offset(0))
        }
    }

    pub fn debug(&self) -> *const Struct_Unnamed13 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn debug_mut(&mut self) -> *mut Struct_Unnamed13 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn mmio(&self) -> *const Struct_Unnamed14 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn mmio_mut(&mut self) -> *mut Struct_Unnamed14 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn hypercall(&self) -> *const Struct_Unnamed15 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn hypercall_mut(&mut self) -> *mut Struct_Unnamed15 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn tpr_access(&self) -> *const Struct_Unnamed16 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn tpr_access_mut(&mut self) -> *mut Struct_Unnamed16 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_sieic(&self) -> *const Struct_Unnamed17 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_sieic_mut(&mut self) -> *mut Struct_Unnamed17 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_reset_flags(&self) -> *const u64 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_reset_flags_mut(&mut self) -> *mut u64 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_ucontrol(&self) -> *const Struct_Unnamed18 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_ucontrol_mut(&mut self) -> *mut Struct_Unnamed18 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }
    pub fn dcr(&self) -> *const Struct_Unnamed19 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn dcr_mut(&mut self) -> *mut Struct_Unnamed19 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn internal(&self) -> *const Struct_Unnamed20 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn internal_mut(&mut self) -> *mut Struct_Unnamed20 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn osi(&self) -> *const Struct_Unnamed21 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn osi_mut(&mut self) -> *mut Struct_Unnamed21 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn papr_hcall(&self) -> *const Struct_Unnamed22 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn papr_hcall_mut(&mut self) -> *mut Struct_Unnamed22 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_tsch(&self) -> *const Struct_Unnamed23 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn s390_tsch_mut(&mut self) -> *mut Struct_Unnamed23 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn epr(&self) -> *const Struct_Unnamed24 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn epr_mut(&mut self) -> *mut Struct_Unnamed24 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

	pub fn system_event(&self) -> *const Struct_Unnamed25 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }

    pub fn system_event_mut(&mut self) -> *mut Struct_Unnamed25 {
        unsafe {
            let raw: *mut u8 = std::mem::transmute(&self._bindgen_data_1_);
            std::mem::transmute(raw.offset(0))
        }
    }
}

impl std::clone::Clone for Run {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::default::Default for Run {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl fmt::Debug for Run {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut s = fmt.debug_struct("Run");
        s.field("request_interrupt_window", &self.request_interrupt_window)
         .field("exit_reason", &self.exit_reason)
         .field("ready_for_interrupt_injection",
                &self.ready_for_interrupt_injection)
         .field("if_flag", &self.if_flag)
         .field("flags", &self.flags)
         .field("cr8", &self.cr8);
        unsafe {
            match self.exit_reason {
                Exit::Unknown => s.field("hw", &*self.hw()),
                Exit::FailEntry => s.field("fail_entry", &*self.fail_entry()),
                Exit::Exception => s.field("ex", &*self.ex()),
                Exit::Io => s.field("io", &*self.io()),
                Exit::Debug => s.field("debug", &*self.debug()),
                Exit::Mmio => s.field("mmio", &*self.mmio()),
                Exit::Hypercall => s.field("hypercall", &*self.hypercall()),
                Exit::TprAccess => s.field("tpr_access", &*self.tpr_access()),
                Exit::S390Sieic => s.field("s390_sieic", &*self.s390_sieic()),
                Exit::S390Reset =>
                    s.field("s390_reset_flags", &*self.s390_reset_flags()),
                Exit::S390Ucontrol =>
                    s.field("s390_ucontrol", &*self.s390_ucontrol()),
                Exit::Dcr => s.field("dcr", &*self.dcr()),
                Exit::Osi => s.field("osi", &*self.osi()),
                Exit::PaprHcall => s.field("papr_hcall", &*self.papr_hcall()),
                Exit::S390Tsch => s.field("s390_tsch", &*self.s390_tsch()),
                Exit::Epr => s.field("epr", &*self.epr()),
                Exit::SystemEvent => s.field("system_event", &*self.system_event()),
                _ => &mut s,
            }
        }
        .finish()
    }
}
