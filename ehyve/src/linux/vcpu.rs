use std;
use libc::{c_int,size_t,close,munmap,c_void};
use vm::VirtualCPU;
use error::*;
use linux::{kvm_create_vcpu, kvm_map_run, kvm_run, kvm_init_vcpu};
use linux::{kvm_get_regs, kvm_get_sregs, Regs, Sregs, KVMFD};
use linux::utils;
use linux::kvm::*;

#[derive(Debug)]
pub struct EhyveCPU
{
	id: u32,
	cpufd: c_int,
	run: *mut Run
}

impl EhyveCPU {
    pub fn new(id: u32, fd: c_int) -> EhyveCPU {
		unsafe {
			let cpufd = kvm_create_vcpu(fd, id as c_int);
			let run = kvm_map_run(KVMFD, cpufd);

			EhyveCPU {
				id: id,
				cpufd: cpufd,
				run: run as *mut Run
			}
		}
	}

	fn get_sregs(&self) -> Result<Sregs> {
		let mut sregs = Sregs::default();
		unsafe {
			let err = kvm_get_sregs(self.cpufd, (&mut sregs) as *mut Sregs);
			if err == -1 {
				return Err(Error::InternalError);
			}
		}

		Ok(sregs)
	}

	fn get_regs(&self) -> Result<Regs> {
		let mut regs = Regs::default();
		unsafe {
			let err = kvm_get_regs(self.cpufd, (&mut regs) as *mut Regs);
			if err == -1 {
				return Err(Error::InternalError);
			}
		}

		Ok(regs)
	}
}

impl VirtualCPU for EhyveCPU {
	fn init(&mut self, entry_point: u64) -> Result<()>
	{
		let ret = unsafe { kvm_init_vcpu(self.cpufd, self.id as c_int, entry_point as size_t) };
		if ret != 0 {
			return Err(Error::InternalError);
		}

		Ok(())
	}

	fn run(&mut self) -> Result<()>
	{
		self.print_registers();

		loop {
			let ret = unsafe { kvm_run(self.cpufd) };

			if ret != 0 {
				return Err(Error::InternalError);
			}

			let reason = unsafe { (*self.run).exit_reason };

			match reason {
				Exit::Io => {
					//debug!("IO Exit");
					unsafe {
						let port = (*(*self.run).io()).port;
						let data = self.run as *const u8;
						let data = data.offset((*(*self.run).io()).data_offset as isize);
						let cstr = std::str::from_utf8(std::slice::from_raw_parts(data, 1)).unwrap();

						self.io_exit(port, cstr.to_string())?;
					}
				},
				Exit::Hlt => {
						debug!("Halt Exit");
				},
				_ => {
					error!("Unknown exit reason: {:?}", reason);
					//self.print_registers();

					return Err(Error::UnknownExitReason(reason));
				}
			}
		}
	}

	fn print_registers(&self)
	{
		utils::show_registers(self.id, &self.get_regs().unwrap(), &self.get_sregs().unwrap());
	}
}

impl Drop for EhyveCPU {
    fn drop(&mut self) {
		debug!("Drop vCPU {}", self.id);

		if self.cpufd >= 0 {
			unsafe {
         		close(self.cpufd);
				munmap(self.run as *mut c_void, std::mem::size_of::<Run>());
			}
		}
    }
}
