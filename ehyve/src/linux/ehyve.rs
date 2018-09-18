//! This file contains the entry point to the Hypervisor. The ehyve utilizes KVM to
//! create a Virtual Machine and load the kernel.

use std;
use libc;
use vm::{Vm, VirtualCPU};
use error::*;
use linux::KVM;
use linux::vcpu::*;
use libkvm::vm::VirtualMachine;
use libkvm::mem::MemorySlot;

pub struct Ehyve {
	vm: VirtualMachine,
	entry_point: u64,
	mem: MmapMemorySlot,
	num_cpus: u32,
	path: String
}

impl Ehyve {
    pub fn new(path: String, mem_size: usize, num_cpus: u32) -> Result<Ehyve> {
		let api = KVM.api_version().unwrap();
		debug!("KVM API version {}", api);

		let vm = KVM.create_vm().unwrap();

		if KVM.check_cap_set_tss_address().unwrap() > 0 {
			debug!("Setting TSS address");
			vm.set_tss_address(0xfffbd000).unwrap();
		}

		let mem = MmapMemorySlot::new(mem_size, 0);
		vm.set_user_memory_region(&mem).unwrap();

		let mut hyve = Ehyve {
			vm: vm,
			entry_point: 0,
			mem: mem,
			num_cpus: num_cpus,
			path: path
		};

		hyve.init()?;

        Ok(hyve)
    }

	fn init(&mut self) -> Result<()> {
		self.init_guest_mem();

		Ok(())
	}
}

impl Vm for Ehyve {
	fn set_entry_point(&mut self, entry: u64)
	{
		self.entry_point = entry;
	}

	fn get_entry_point(&self) -> u64
	{
		self.entry_point
	}

	fn num_cpus(&self) -> u32 {
		self.num_cpus
	}

	fn guest_mem(&self) -> (*mut u8, usize) {
		(self.mem.host_address() as *mut u8, self.mem.memory_size())
	}

	fn kernel_path(&self) -> &str {
			&self.path
	}

	fn create_cpu(&self, id: u32) -> Result<Box<VirtualCPU>> {
		Ok(Box::new(EhyveCPU::new(id, self.vm.create_vcpu().unwrap())))
	}
}

impl Drop for Ehyve {
    fn drop(&mut self) {
        debug!("Drop virtual machine");
    }
}

unsafe impl Send for Ehyve {}
unsafe impl Sync for Ehyve {}

struct MmapMemorySlot {
    memory_size: usize,
    guest_address: u64,
    host_address: *mut libc::c_void,
}

impl MmapMemorySlot {
    pub fn new(memory_size: usize, guest_address: u64) -> MmapMemorySlot {
        let host_address = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                memory_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };

        if host_address == libc::MAP_FAILED {
            panic!("mmapp failed with: {}", unsafe {
                *libc::__errno_location()
            });
        }

        MmapMemorySlot {
            memory_size: memory_size,
            guest_address: guest_address,
            host_address,
        }
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.host_address as *mut u8, self.memory_size) }
    }
}

impl MemorySlot for MmapMemorySlot {
    fn slot_id(&self) -> u32 {
        0
    }

    fn flags(&self) -> u32 {
        0
    }

    fn memory_size(&self) -> usize {
        self.memory_size
    }

    fn guest_address(&self) -> u64 {
        self.guest_address
    }

    fn host_address(&self) -> u64 {
        self.host_address as u64
    }
}

impl Drop for MmapMemorySlot {
    fn drop(&mut self) {
        let result = unsafe { libc::munmap(self.host_address, self.memory_size) };
        if result != 0 {
            panic!("munmap failed with: {}", unsafe {
                *libc::__errno_location()
            });
        }
    }
}
