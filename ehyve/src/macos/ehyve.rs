use std;
use std::mem;
use vm::{Vm, VirtualCPU};
use consts::*;
use macos::error::*;
use macos::vcpu::*;
use hypervisor::{create_vm,map_mem,unmap_mem,MemPerm};
use aligned_alloc::*;

#[derive(Debug, Clone)]
pub struct Ehyve {
	entry_point: u64,
	mem_size: usize,
	guest_mem: *mut (),
	num_cpus: u32,
	path: String
}

impl Ehyve {
    pub fn new(path: String, mem_size: usize, num_cpus: u32) -> Result<Ehyve> {
		let mem = aligned_alloc(mem_size, PAGE_SIZE);

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

		debug!("Map guest menory...");
		unsafe {
			map_mem(std::slice::from_raw_parts(self.guest_mem as *const u8,
				mem::size_of::<&[u8]>()), 0, &MemPerm::ExecAndWrite).or_else(to_error)?;
		}

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

		unsafe { aligned_free(self.guest_mem); }
    }
}

unsafe impl Send for Ehyve {}
unsafe impl Sync for Ehyve {}
