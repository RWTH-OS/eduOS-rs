pub const DEFAULT_GUEST_SIZE: usize = 32*1024*1024;
pub const PAGE_SIZE: usize		= 0x1000;
pub const GDT_KERNEL_CODE: u16	= 1;
pub const GDT_KERNEL_DATA: u16	= 2;
pub const APIC_DEFAULT_BASE: u64 = 0xfee00000;
pub const BOOT_GDT: u64 		= 0x1000;
pub const BOOT_GDT_NULL: u64 	= 0;
pub const BOOT_GDT_CODE: u64 	= 1;
pub const BOOT_GDT_DATA: u64 	= 2;
pub const BOOT_GDT_MAX: u64 	= 3;
pub const BOOT_PML4: u64 		= 0x10000;
pub const BOOT_PDPTE: u64       = 0x11000;
pub const BOOT_PDE: u64         = 0x12000;
pub const EFER_SCE: u64			= (1 << 0);		/* System Call Extensions */
pub const EFER_LME: u64			= (1 << 8);		/* Long mode enable */
pub const EFER_LMA: u64			= (1 << 10);	/* Long mode active (read-only) */
pub const EFER_NXE: u64			= (1 << 11);	/* PTE No-Execute bit enable */
pub const COM_PORT: u16			= 0x3f8;
pub const SHUTDOWN_PORT: u16	= 0xf4;

/*
 * Intel long mode page directory/table entries
 */
pub const X86_PDPT_P: u64       = (1u64 << 0);  /* Present */
pub const X86_PDPT_RW: u64      = (1u64 << 1);  /* writeable */
pub const X86_PDPT_PS: u64      = (1u64 << 7);  /* Page size */

pub const GUEST_PAGE_SIZE: u64  = 0x200000;     /* 2 MB pages in guest */
