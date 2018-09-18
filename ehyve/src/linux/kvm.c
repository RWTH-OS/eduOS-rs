#define _GNU_SOURCE

#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/kvm.h>
#include <sys/ioctl.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <err.h>
#ifdef HAVE_MSR_INDEX_H
#include <asm/msr-index.h>
#else
/* x86-64 specific MSRs */
#define MSR_EFER                0xc0000080 /* extended feature register */
#define MSR_STAR                0xc0000081 /* legacy mode SYSCALL target */
#define MSR_LSTAR               0xc0000082 /* long mode SYSCALL target */
#define MSR_CSTAR               0xc0000083 /* compat mode SYSCALL target */
#define MSR_SYSCALL_MASK        0xc0000084 /* EFLAGS mask for syscall */
#define MSR_FS_BASE             0xc0000100 /* 64bit FS base */
#define MSR_GS_BASE             0xc0000101 /* 64bit GS base */
#define MSR_KERNEL_GS_BASE      0xc0000102 /* SwapGS GS shadow */
#define MSR_TSC_AUX             0xc0000103 /* Auxiliary TSC */

#define MSR_IA32_CR_PAT         0x00000277
#define MSR_PEBS_FRONTEND       0x000003f7

#define MSR_IA32_POWER_CTL      0x000001fc

#define MSR_IA32_MC0_CTL        0x00000400
#define MSR_IA32_MC0_STATUS     0x00000401
#define MSR_IA32_MC0_ADDR       0x00000402
#define MSR_IA32_MC0_MISC       0x00000403

#define MSR_IA32_SYSENTER_CS    0x00000174
#define MSR_IA32_SYSENTER_ESP   0x00000175
#define MSR_IA32_SYSENTER_EIP   0x00000176

#define MSR_IA32_APICBASE       0x0000001b
#define MSR_IA32_APICBASE_BSP   (1<<8)
#define MSR_IA32_APICBASE_ENABLE (1<<11)
#define MSR_IA32_APICBASE_BASE  (0xfffff<<12)
#define MSR_IA32_MISC_ENABLE    0x000001a0
#define MSR_IA32_TSC            0x00000010

/* EFER bits: */
#define _EFER_SCE               0  /* SYSCALL/SYSRET */
#define _EFER_LME               8  /* Long mode enable */
#define _EFER_LMA               10 /* Long mode active (read-only) */
#define _EFER_NX                11 /* No execute enable */
#define _EFER_SVME              12 /* Enable virtualization */
#define _EFER_LMSLE             13 /* Long Mode Segment Limit Enable */
#define _EFER_FFXSR             14 /* Enable Fast FXSAVE/FXRSTOR */

#define EFER_SCE                (1<<_EFER_SCE)
#define EFER_LME                (1<<_EFER_LME)
#define EFER_LMA                (1<<_EFER_LMA)
#define EFER_NX                 (1<<_EFER_NX)
#define EFER_SVME               (1<<_EFER_SVME)
#define EFER_LMSLE              (1<<_EFER_LMSLE)
#define EFER_FFXSR              (1<<_EFER_FFXSR)
#endif
#include "kvm.h"

static bool cap_tsc_deadline = false;
static struct kvm_cpuid2 *kvm_cpuid = NULL;

/// Filter CPUID functions that are not supported by the hypervisor and enable
/// features according to our needs.
static void filter_cpuid(struct kvm_cpuid2 *kvm_cpuid)
{
	uint32_t i;

        for (i = 0; i < kvm_cpuid->nent; i++) {
                struct kvm_cpuid_entry2 *entry = &kvm_cpuid->entries[i];

                switch (entry->function) {
                case 1:
                        // CPUID to define basic cpu features
                        entry->ecx |= (1U << 31); // propagate that we are running on a hypervisor
                        if (cap_tsc_deadline)
                                entry->ecx |= (1U << 24); // enable TSC deadline feature
                        entry->edx |= (1U <<  5); // enable msr support
                        break;

                case CPUID_FUNC_PERFMON:
                        // disable it
                        entry->eax      = 0x00;
                        break;

                default:
                        // Keep the CPUID function as-is
                        break;
                };
        }
}

int kvm_init(void) {
	int fd = open("/dev/kvm", O_RDWR | O_CLOEXEC);

	if (fd < 0)
		err(1, "Could not open: /dev/kvm");

	/* Make sure we have the stable version of the API */
	int kvm_api_version = kvm_ioctl(fd, KVM_GET_API_VERSION, NULL);
	if (kvm_api_version != API_VERSION) {
		err(1, "KVM: API version is %d, uhyve requires version 12", kvm_api_version);
	}

	return fd;
}

int kvm_create_vm(int fd, int flags) {
	int vmfd = kvm_ioctl(fd, KVM_CREATE_VM, flags);

	cap_tsc_deadline = kvm_ioctl(vmfd, KVM_CHECK_EXTENSION, KVM_CAP_TSC_DEADLINE_TIMER) <= 0 ? false : true;

	// allocate space for cpuid we get from KVM
        if (!kvm_cpuid) {
		unsigned int max_entries = 100;

		kvm_cpuid = calloc(1, sizeof(*kvm_cpuid) + (max_entries * sizeof(kvm_cpuid->entries[0])));
        	kvm_cpuid->nent = max_entries;

		kvm_ioctl(fd, KVM_GET_SUPPORTED_CPUID, kvm_cpuid);

		filter_cpuid(kvm_cpuid);
	}

	return vmfd;
}

uint8_t* kvm_init_vm(int vmfd, size_t guest_size) {
	uint8_t* guest_mem = NULL;
	uint64_t identity_base = 0xfffbc000;

        if (kvm_ioctl(vmfd, KVM_CHECK_EXTENSION, KVM_CAP_SYNC_MMU) > 0) {
                /* Allows up to 16M BIOSes. */
                identity_base = 0xfeffc000;

                kvm_ioctl(vmfd, KVM_SET_IDENTITY_MAP_ADDR, &identity_base);
        }
        kvm_ioctl(vmfd, KVM_SET_TSS_ADDR, identity_base + 0x1000);

        /*
         * Allocate page-aligned guest memory.
         *
         * TODO: support of huge pages
         */
        if (guest_size < KVM_32BIT_GAP_START) {
                guest_mem = mmap(NULL, guest_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
                if (guest_mem == MAP_FAILED)
                        err(1, "mmap failed");
        } else {
                guest_size += KVM_32BIT_GAP_SIZE;
                guest_mem = mmap(NULL, guest_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
                if (guest_mem == MAP_FAILED)
                        err(1, "mmap failed");

                /*
                 * We mprotect the gap PROT_NONE so that if we accidently write to it, we will know.
                 */
                mprotect(guest_mem + KVM_32BIT_GAP_START, KVM_32BIT_GAP_SIZE, PROT_NONE);
        }

	madvise(guest_mem, guest_size, MADV_HUGEPAGE);

	struct kvm_userspace_memory_region kvm_region = {
                .slot = 0,
                .guest_phys_addr = GUEST_OFFSET,
                .memory_size = guest_size,
                .userspace_addr = (uint64_t) guest_mem,
                .flags = 0,
        };

        if (guest_size <= KVM_32BIT_GAP_START - GUEST_OFFSET) {
                kvm_ioctl(vmfd, KVM_SET_USER_MEMORY_REGION, &kvm_region);
        } else {
                kvm_region.memory_size = KVM_32BIT_GAP_START - GUEST_OFFSET;
                kvm_ioctl(vmfd, KVM_SET_USER_MEMORY_REGION, &kvm_region);

                kvm_region.slot = 1;
                kvm_region.guest_phys_addr = KVM_32BIT_GAP_START + KVM_32BIT_GAP_SIZE;
                kvm_region.userspace_addr = (uint64_t) guest_mem + KVM_32BIT_GAP_START + KVM_32BIT_GAP_SIZE;
                kvm_region.memory_size = guest_size - KVM_32BIT_GAP_SIZE - KVM_32BIT_GAP_START + GUEST_OFFSET;
                kvm_ioctl(vmfd, KVM_SET_USER_MEMORY_REGION, &kvm_region);
        }

	return guest_mem;
}

int kvm_create_vcpu(int fd, int vcpu_id) {
	return kvm_ioctl(fd, KVM_CREATE_VCPU, vcpu_id);
}

struct kvm_run* kvm_map_run(int fd, int vcpufd)
{
	size_t mmap_size = (size_t) kvm_ioctl(fd, KVM_GET_VCPU_MMAP_SIZE, NULL);

	/* Map the shared kvm_run structure and following data. */
	if (mmap_size < sizeof(struct kvm_run))
		err(1, "KVM: invalid VCPU_MMAP_SIZE: %zd", mmap_size);

	// TODO: unmap run, if we detroy the VM
	struct kvm_run* run = mmap(NULL, mmap_size, PROT_READ | PROT_WRITE, MAP_SHARED, vcpufd, 0);
	if (run == MAP_FAILED)
		err(1, "KVM: VCPU mmap failed");
	run->apic_base = APIC_DEFAULT_BASE;

	return run;
}

static void setup_system_64bit(struct kvm_sregs *sregs)
{
	sregs->cr3 = BOOT_PML4;
        sregs->cr0 |= X86_CR0_PE | X86_CR0_PG;
        sregs->cr4 |= X86_CR4_PAE;
        sregs->efer |= EFER_LME | EFER_LMA;
}

static void setup_system_gdt(struct kvm_sregs *sregs)
{
        struct kvm_segment data_seg, code_seg;

        sregs->gdt.base = BOOT_GDT;
        sregs->gdt.limit = (sizeof(uint64_t) * BOOT_GDT_MAX) - 1;

	uint64_t code_ent = GDT_ENTRY(0xA09B, 0, 0xFFFFF);
	uint64_t data_ent = GDT_ENTRY(0xC093, 0, 0xFFFFF);
        GDT_TO_KVM_SEGMENT(code_seg, BOOT_GDT_CODE, code_ent);
        GDT_TO_KVM_SEGMENT(data_seg, BOOT_GDT_DATA, data_ent);

        sregs->cs = code_seg;
        sregs->ds = data_seg;
        sregs->es = data_seg;
        sregs->fs = data_seg;
        sregs->gs = data_seg;
        sregs->ss = data_seg;

	sregs->apic_base = APIC_DEFAULT_BASE;
}

static void setup_system(int vcpufd, uint32_t id)
{
        static struct kvm_sregs sregs;

        // all cores use the same startup code
        // => all cores use the same sregs
        // => only the boot processor has to initialize sregs
        if (id == 0) {
                kvm_ioctl(vcpufd, KVM_GET_SREGS, &sregs);

                /* Set all cpu/mem system structures */
                setup_system_gdt(&sregs);
                setup_system_64bit(&sregs);
        }

        kvm_ioctl(vcpufd, KVM_SET_SREGS, &sregs);
}

int kvm_init_vcpu(int vcpufd, int cpuid, uint64_t elf_entry) {
	struct kvm_regs regs = {
		.rip = elf_entry,		// entry point to HermitCore
		.rsp = 0x200000 - 0x1000,	// temporary stack to boot the kernel
		.rflags = 0x2,			// POR value required by x86 architecture
	};
	struct kvm_mp_state mp_state = { KVM_MP_STATE_RUNNABLE };
	struct {
		struct kvm_msrs info;
		struct kvm_msr_entry entries[MAX_MSR_ENTRIES];
	} msr_data;
	struct kvm_msr_entry *msrs = msr_data.entries;

	// set cpu features
        kvm_ioctl(vcpufd, KVM_SET_CPUID2, kvm_cpuid);

	// be sure that the multiprocessor is runable
	kvm_ioctl(vcpufd, KVM_SET_MP_STATE, &mp_state);

	// enable fast string operations
	msrs[0].index = MSR_IA32_MISC_ENABLE;
	msrs[0].data = 1;
	msr_data.info.nmsrs = 1;
	kvm_ioctl(vcpufd, KVM_SET_MSRS, &msr_data);

	// only one core is able to enter startup code
	// => the wait for the predecessor core
	//while (*((volatile uint32_t*) (mboot + 0x20)) < cpuid)
	//	pthread_yield();
	//*((volatile uint32_t*) (mboot + 0x30)) = cpuid;

	/* Setup registers and memory. */
	setup_system(vcpufd, cpuid);
	kvm_ioctl(vcpufd, KVM_SET_REGS, &regs);

	return 0;
}

int kvm_run(int fd) {
	int ret;

retry:
	ret = kvm_ioctl(fd, KVM_RUN, 0);

	if(ret == -1) {
        	switch(errno) {
                case EINTR:
                        goto retry;

                case EFAULT: {
                        struct kvm_regs regs;
                        kvm_ioctl(fd, KVM_GET_REGS, &regs);
                        err(1, "KVM: host/guest translation fault: rip=0x%llx", regs.rip);
                }

                default:
                        err(1, "KVM: ioctl KVM_RUN failed");
                        break;
                }
        }

	return ret;
}

int kvm_get_regs(int fd, struct kvm_regs *regs) {
  return kvm_ioctl(fd, KVM_GET_REGS, regs);
}

int kvm_set_regs(int fd, const struct kvm_regs *regs) {
  return kvm_ioctl(fd, KVM_SET_REGS, regs);
}

int kvm_get_sregs(int fd, struct kvm_sregs *sregs) {
  return kvm_ioctl(fd, KVM_GET_SREGS, sregs);
}

int kvm_set_sregs(int fd, const struct kvm_sregs *sregs) {
  return kvm_ioctl(fd, KVM_SET_SREGS, sregs);
}
