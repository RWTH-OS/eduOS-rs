use std;
use libc;
use libc::c_void;
use vm::{Vm, VirtualCPU};
use error::*;
use macos::vcpu::*;
use hypervisor::{create_vm,map_mem,unmap_mem,MemPerm};

#[derive(Debug)]
pub struct Ehyve {
	entry_point: u64,
	mem_size: usize,
	guest_mem: *mut c_void,
	num_cpus: u32,
	path: String
}

impl Ehyve {
	pub fn new(path: String, mem_size: usize, num_cpus: u32) -> Result<Ehyve> {
		let mem = unsafe {
			libc::mmap(
				std::ptr::null_mut(),
				mem_size,
				libc::PROT_READ | libc::PROT_WRITE,
				libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_NORESERVE,
				-1,
				0,
			)
		};

		if mem == libc::MAP_FAILED {
			error!("mmap failed with");
			return Err(Error::NotEnoughMemory);
		}

		debug!("Allocate memory for the guest at 0x{:x}", mem as usize);

		let mut hyve = Ehyve {
			entry_point: 0,
			mem_size: mem_size,
			guest_mem: mem,
			num_cpus: num_cpus,
			path: path
		};

		hyve.init()?;

		Ok(hyve)
	}

	fn init(&mut self) -> Result<()> {
		debug!("Create VM...");
		create_vm().or_else(to_error)?;

		debug!("Map guest memory...");
		unsafe {
			map_mem(std::slice::from_raw_parts(self.guest_mem as *mut u8, self.mem_size),
			0, &MemPerm::ExecAndWrite).or_else(to_error)?;
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
		(self.guest_mem as *mut u8, self.mem_size)
	}

	fn kernel_path(&self) -> &str {
		&self.path
	}

	fn create_cpu(&self, id: u32) -> Result<Box<VirtualCPU>> {
		Ok(Box::new(EhyveCPU::new(id)))
	}
}

impl Drop for Ehyve {
	fn drop(&mut self) {
		debug!("Drop virtual machine");

		unmap_mem(0, self.mem_size).unwrap();

		unsafe { libc::munmap(self.guest_mem, self.mem_size); }
	}
}

unsafe impl Send for Ehyve {}
unsafe impl Sync for Ehyve {}
