#ifndef __KVM_H__
#define __UKVM_H__

#ifndef _BITUL

#ifdef __ASSEMBLY__
#define _AC(X,Y)	X
#define _AT(T,X)	X
#else
#define __AC(X,Y)	(X##Y)
#define _AC(X,Y)	__AC(X,Y)
#define _AT(T,X)	((T)(X))
#endif

#define _BITUL(x)	(_AC(1,UL) << (x))
#define _BITULL(x)	(_AC(1,ULL) << (x))

#endif

// Page is present
#define PG_PRESENT              (1 << 0)
// Page is read- and writable
#define PG_RW                   (1 << 1)
// Page is addressable from userspace
#define PG_USER                 (1 << 2)
// Page write through is activated
#define PG_PWT                  (1 << 3)
// Page cache is disabled
#define PG_PCD                  (1 << 4)
// Page was recently accessed (set by CPU)
#define PG_ACCESSED             (1 << 5)
// Page is dirty due to recent write-access (set by CPU)
#define PG_DIRTY                (1 << 6)
// Huge page: 4MB (or 2MB, 1GB)
#define PG_PSE                  (1 << 7)
// Page attribute table
#define PG_PAT                  PG_PSE
// Global TLB entry (Pentium Pro and later)
#define PG_GLOBAL               (1 << 8)
// This table is a self-reference and should skipped by page_map_copy()
#define PG_SELF                 (1 << 9)

/// Disable execution for this page
#define PG_XD                   (1L << 63)

#define CPUID_FUNC_PERFMON	0x0A
#define MAX_MSR_ENTRIES		25
#define IOAPIC_DEFAULT_BASE	0xfec00000
#define APIC_DEFAULT_BASE	0xfee00000
#define GUEST_OFFSET		0x0
#define API_VERSION		12
#define BOOT_GDT		0x1000
#define BOOT_INFO		0x2000
#define BOOT_PML4		0x10000
#define BOOT_PDPTE		0x11000
#define BOOT_PDE		0x12000
#define BOOT_GDT_NULL		0
#define BOOT_GDT_CODE		1
#define BOOT_GDT_DATA		2
#define BOOT_GDT_MAX		3
#define KVM_32BIT_MAX_MEM_SIZE	(1ULL << 32)
#define KVM_32BIT_GAP_SIZE	(768 << 20)
#define KVM_32BIT_GAP_START	(KVM_32BIT_MAX_MEM_SIZE - KVM_32BIT_GAP_SIZE)

#define kvm_ioctl(fd, cmd, arg) ({ \
        const int ret = ioctl(fd, cmd, arg); \
        if(ret == -1) \
                err(1, "KVM: ioctl " #cmd " failed"); \
        ret; \
        })


/*
 * EFLAGS bits
 */
#define X86_EFLAGS_CF	0x00000001 /* Carry Flag */

/*
 * Basic CPU control in CR0
 */
#define X86_CR0_PE_BIT		0 /* Protection Enable */
#define X86_CR0_PE		_BITUL(X86_CR0_PE_BIT)
#define X86_CR0_PG_BIT		31 /* Paging */
#define X86_CR0_PG		_BITUL(X86_CR0_PG_BIT)

/*
 * Intel CPU features in CR4
 */
#define X86_CR4_PAE_BIT		5 /* enable physical address extensions */
#define X86_CR4_PAE		_BITUL(X86_CR4_PAE_BIT)

/*
 * Intel long mode page directory/table entries
 */
#define X86_PDPT_P_BIT          0 /* Present */
#define X86_PDPT_P              _BITUL(X86_PDPT_P_BIT)
#define X86_PDPT_RW_BIT         1 /* Writable */
#define X86_PDPT_RW             _BITUL(X86_PDPT_RW_BIT)
#define X86_PDPT_PS_BIT         7 /* Page size */
#define X86_PDPT_PS             _BITUL(X86_PDPT_PS_BIT)

/*
 * GDT and KVM segment manipulation
 */

#define GDT_DESC_OFFSET(n) ((n) * 0x8)

#define GDT_GET_BASE(x) (                      \
    (((x) & 0xFF00000000000000) >> 32) |       \
    (((x) & 0x000000FF00000000) >> 16) |       \
    (((x) & 0x00000000FFFF0000) >> 16))

#define GDT_GET_LIMIT(x) (__u32)(                                      \
                                 (((x) & 0x000F000000000000) >> 32) |  \
                                 (((x) & 0x000000000000FFFF)))

/* Constructor for a conventional segment GDT (or LDT) entry */
/* This is a macro so it can be used in initializers */
#define GDT_ENTRY(flags, base, limit)               \
    ((((base)  & _AC(0xff000000, ULL)) << (56-24)) | \
     (((flags) & _AC(0x0000f0ff, ULL)) << 40) |      \
     (((limit) & _AC(0x000f0000, ULL)) << (48-16)) | \
     (((base)  & _AC(0x00ffffff, ULL)) << 16) |      \
     (((limit) & _AC(0x0000ffff, ULL))))

#define GDT_GET_G(x)   (__u8)(((x) & 0x0080000000000000) >> 55)
#define GDT_GET_DB(x)  (__u8)(((x) & 0x0040000000000000) >> 54)
#define GDT_GET_L(x)   (__u8)(((x) & 0x0020000000000000) >> 53)
#define GDT_GET_AVL(x) (__u8)(((x) & 0x0010000000000000) >> 52)
#define GDT_GET_P(x)   (__u8)(((x) & 0x0000800000000000) >> 47)
#define GDT_GET_DPL(x) (__u8)(((x) & 0x0000600000000000) >> 45)
#define GDT_GET_S(x)   (__u8)(((x) & 0x0000100000000000) >> 44)
#define GDT_GET_TYPE(x)(__u8)(((x) & 0x00000F0000000000) >> 40)

#define GDT_TO_KVM_SEGMENT(seg, gdt_table, sel) \
    do {                                        \
        __u64 gdt_ent = gdt_table[sel];         \
        seg.base = GDT_GET_BASE(gdt_ent);       \
        seg.limit = GDT_GET_LIMIT(gdt_ent);     \
        seg.selector = sel * 8;                 \
        seg.type = GDT_GET_TYPE(gdt_ent);       \
        seg.present = GDT_GET_P(gdt_ent);       \
        seg.dpl = GDT_GET_DPL(gdt_ent);         \
        seg.db = GDT_GET_DB(gdt_ent);           \
        seg.s = GDT_GET_S(gdt_ent);             \
        seg.l = GDT_GET_L(gdt_ent);             \
        seg.g = GDT_GET_G(gdt_ent);             \
        seg.avl = GDT_GET_AVL(gdt_ent);         \
    } while (0)

#endif
