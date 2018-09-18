//! This file contains the entry point to the Hypervisor. The ehyve utilizes KVM to
//! create a Virtual Machine and load the kernel.

use libc::{c_int,close,c_void,munmap};
use vm::{Vm, VirtualCPU};
use error::*;
use linux::vcpu::*;
use linux::{KVMFD, kvm_create_vm, kvm_init, kvm_init_vm};
use x86::bits64::segmentation::*;

struct Gdt {
	entries: [SegmentDescriptor; 3]
}

#[derive(Debug)]
pub struct Ehyve {
	vmfd: c_int,
	entry_point: u64,
	guest_size: usize,
	guest_mem: *mut c_void,
	num_cpus: u32,
	path: String
}

impl Ehyve {
    pub fn new(path: String, mem_size: usize, num_cpus: u32) -> Result<Ehyve> {
		unsafe {
			if KVMFD < 0 {
				KVMFD = kvm_init();

				if KVMFD < 0 {
					return Err(Error::KVMInitFailed);
				}
			}
		}

		let mut hyve = Ehyve {
			vmfd: unsafe { kvm_create_vm(KVMFD, 0) },
			entry_point: 0,
			guest_size: mem_size,
			guest_mem: 0 as *mut c_void,
			num_cpus: num_cpus,
			path: path
		};

		if hyve.vmfd < 0 {
				return Err(Error::KVMUnableToCreateVM);
		}

		hyve.init()?;

        Ok(hyve)
    }

	fn init(&mut self) -> Result<()> {
		debug!("Map guest menory...");
		self.guest_mem = unsafe { kvm_init_vm(self.vmfd, self.guest_size) };

		if self.guest_mem == 0 as *mut c_void {
			return Err(Error::NotEnoughMemory);
		}

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
		(self.guest_mem as *mut u8, self.guest_size)
	}

	fn kernel_path(&self) -> &str {
			&self.path
	}

	fn create_cpu(&self, id: u32) -> Result<Box<VirtualCPU>> {
		Ok(Box::new(EhyveCPU::new(id, self.vmfd)))
	}
}

impl Drop for Ehyve {
    fn drop(&mut self) {
        debug!("Drop virtual machine");

		if self.vmfd >= 0 {
			unsafe { close(self.vmfd) };
		}

		if self.guest_mem > 0 as *mut c_void {
			unsafe { munmap(self.guest_mem, self.guest_size) };
		}
    }
}

unsafe impl Send for Ehyve {}
unsafe impl Sync for Ehyve {}
