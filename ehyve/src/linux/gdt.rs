//! This module wraps the structure of the GDT and helps to extract certain information about the
//! GDT. The Global Descriptor Table contains information about the memory structure used by the
//! X86 family.

use linux::kvm::*;

/// Used segments in order
pub const BOOT_NULL: isize = 0;
pub const BOOT_CODE: isize = 1;
pub const BOOT_DATA: isize = 2;
pub const BOOT_MAX: usize  = 3;

/// This struct breaks the access part of a GDT entry down into functions
pub struct AccessBits {
    access: u8
}

impl AccessBits {
    pub fn present(&self) -> u8 {
        (self.access & 0b10000000) >> 7
    }

    pub fn privilege(&self) -> u8 {
        (self.access & 0b01100000) >> 5
    }

    pub fn s(&self) -> u8 {
        (self.access & 0b00010000) >> 4
    }

    pub fn kind(&self) -> u8 {
        (self.access & 0b00001111)
    }

    #[inline(always)]
    pub fn apply_to_kvm(&self, seg: &mut kvm_segment) {
        seg.present = self.present();
        seg.dpl = self.privilege();
        seg.s = self.s();
        seg.type_ = self.kind();
    }
}

pub struct FlagBits {
    flags: u8
}

impl FlagBits {
    pub fn granularity(&self) -> u8 {
        (self.flags & 0b00001000) >> 3
    }

    pub fn size(&self) -> u8 {
        (self.flags & 0b00000100) >> 2
    }

    pub fn desc_x86_64(&self) -> u8 {
        (self.flags & 0b00000010) >> 1
    }

    pub fn sz_x86_64(&self) -> u8 {
        (self.flags & 0b00000001)
    }

    #[inline(always)]
    pub fn apply_to_kvm(&self, seg: &mut kvm_segment) {
        seg.g = self.granularity();
        seg.db = self.size();
        seg.l = self.desc_x86_64();
        seg.avl = self.sz_x86_64();
    }
}

/// This struct defines the arrangment one GDT entry in memory
pub struct Entry {
    pub limit_l: u16,
    pub offset_l: u16,
    pub offset_m: u8,
    pub access: u8,
    pub flags_limit_h: u8,
    pub offset_h: u8
}

impl Entry {
    pub fn new(flags: u16, offset: u32, limit: u32) -> Entry {
        Entry {
            limit_l: limit as u16,
            offset_l: offset as u16,
            offset_m: (offset >> 16) as u8,
            access: flags as u8,
            flags_limit_h: (limit >> 16) as u8 & 0x0F | (flags >> 8) as u8 & 0xF0,
            offset_h: (offset >> 24) as u8
        }
    }

    // returns the maximum addressable unit
    pub fn get_limit(&self) -> u32 {
        ((self.flags_limit_h & 0x0F) as u32) << 16 | (self.limit_l as u32)
    }

    // returns the offset of this segment
    pub fn get_offset(&self) -> u32 {
        (self.offset_h as u32) << 24 | (self.offset_m as u32) << 16 | (self.offset_l as u32)
    }

    // returns the access bits
    pub fn get_access(&self) -> AccessBits {
        AccessBits { access: self.access }
    }

    // return the flags bits
    pub fn get_flags(&self) -> FlagBits {
        FlagBits { flags: (self.flags_limit_h & 0xF0) >> 4 }
    }

    // convert the struct to an unsigned integer
    pub fn as_u64(&self) -> u64 {
        (self.offset_h as u64)      << 56 |
        (self.flags_limit_h as u64) << 48 |
        (self.access as u64)        << 40 |
        (self.offset_m as u64)      << 32 |
        (self.offset_l as u64)      << 16 |
        (self.limit_l as u64)       <<  0
    }

    #[inline(always)]
    pub fn apply_to_kvm(&self, sel: isize, seg: &mut kvm_segment) {
        seg.base = self.get_offset() as u64;
        seg.limit = self.get_limit();
        seg.selector = sel as u16 * 8;

        self.get_access().apply_to_kvm(seg);
        self.get_flags().apply_to_kvm(seg);

    }

}
