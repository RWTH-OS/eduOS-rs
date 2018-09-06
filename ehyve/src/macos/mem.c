#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <string.h>
#include <errno.h>

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

/* Constructor for a conventional segment GDT (or LDT) entry */
/* This is a macro so it can be used in initializers */
#define GDT_ENTRY(flags, base, limit)               \
    ((((base)  & _AC(0xff000000, ULL)) << (56-24)) | \
     (((flags) & _AC(0x0000f0ff, ULL)) << 40) |      \
     (((limit) & _AC(0x000f0000, ULL)) << (48-16)) | \
     (((base)  & _AC(0x00ffffff, ULL)) << 16) |      \
     (((limit) & _AC(0x0000ffff, ULL))))
     
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

#define GUEST_OFFSET		0x0
#define GUEST_PAGE_SIZE		0x200000   /* 2 MB pages in guest */
#define BOOT_GDT		0x1000
#define BOOT_INFO		0x2000
#define BOOT_PML4		0x10000
#define BOOT_PDPTE		0x11000
#define BOOT_PDE		0x12000
#define BOOT_GDT_NULL		0
#define BOOT_GDT_CODE		1
#define BOOT_GDT_DATA		2
#define BOOT_GDT_MAX		3

int setup_guest_mem(uint8_t *mem)
{
	if (!mem)
		return -EFAULT;

	uint64_t *gdt = (uint64_t *) (mem + BOOT_GDT);
	uint64_t *pml4 = (uint64_t *) (mem + BOOT_PML4);
	uint64_t *pdpte = (uint64_t *) (mem + BOOT_PDPTE);
	uint64_t *pde = (uint64_t *) (mem + BOOT_PDE);
	uint64_t paddr;

	/*
	 * For simplicity we currently use 2MB pages and only a single
	 * PML4/PDPTE/PDE.
	 */

	memset(pml4, 0x00, 4096);
	memset(pdpte, 0x00, 4096);
	memset(pde, 0x00, 4096);

	*pml4 = BOOT_PDPTE | (X86_PDPT_P | X86_PDPT_RW);
	*pdpte = BOOT_PDE | (X86_PDPT_P | X86_PDPT_RW);
	for (paddr = 0; paddr < 0x20000000ULL; paddr += GUEST_PAGE_SIZE, pde++)
		*pde = paddr | (X86_PDPT_P | X86_PDPT_RW | X86_PDPT_PS);

	/* flags, base, limit */
	gdt[BOOT_GDT_NULL] = GDT_ENTRY(0, 0, 0);
	gdt[BOOT_GDT_CODE] = GDT_ENTRY(0xA09B, 0, 0xFFFFF);
	gdt[BOOT_GDT_DATA] = GDT_ENTRY(0xC093, 0, 0xFFFFF);

	return 0;
}
