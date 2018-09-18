use std;
use vm::VirtualCPU;
use error::*;
use consts::*;
use libkvm::linux::kvm_bindings::*;
use libkvm::vcpu;
use linux::KVM;
use x86::shared::control_regs::*;

const CPUID_EXT_HYPERVISOR: u32 = 1 << 31;

pub struct EhyveCPU
{
	id: u32,
	vcpu: vcpu::VirtualCPU
}

impl EhyveCPU {
    pub fn new(id: u32, vcpu: vcpu::VirtualCPU) -> EhyveCPU {
		EhyveCPU {
			id: id,
			vcpu: vcpu
		}
	}

	fn setup_cpuid(&self) {
		let mut kvm_cpuid_entries = KVM.get_supported_cpuid().unwrap();

		let i = kvm_cpuid_entries
		.iter()
		.position(|&r| r.function == 0x40000000)
		.unwrap();

		let mut id_reg_values: [u32; 3] = [0; 3];
		let id = "libkvm\0";
		unsafe {
			std::ptr::copy_nonoverlapping(id.as_ptr(), id_reg_values.as_mut_ptr() as *mut u8, id.len());
		}
		kvm_cpuid_entries[i].ebx = id_reg_values[0];
		kvm_cpuid_entries[i].ecx = id_reg_values[1];
		kvm_cpuid_entries[i].edx = id_reg_values[2];

		let i = kvm_cpuid_entries
				.iter()
				.position(|&r| r.function == 1)
				.unwrap();

		kvm_cpuid_entries[i].ecx |= CPUID_EXT_HYPERVISOR;

		self.vcpu.set_cpuid(&kvm_cpuid_entries).unwrap();
	}

	fn setup_msrs(&self) {
	    let msr_list = KVM.get_msr_index_list().unwrap();

	    let msr_entries = msr_list
	        .iter()
	        .map(|i| kvm_msr_entry {
	            index: *i,
	            data: 0,
	            ..Default::default()
	        })
	        .collect::<Vec<_>>();

	    self.vcpu.set_msrs(&msr_entries).unwrap();
	}

	fn setup_long_mode(&self, entry_point: u64) {
		debug!("Setup long mode");

		let mut sregs = self.vcpu.get_kvm_sregs().unwrap();

		let cr0 = (CR0_PROTECTED_MODE | CR0_ENABLE_PAGING | CR0_EXTENSION_TYPE | CR0_NUMERIC_ERROR).bits() as u64;
		let cr4 = CR4_ENABLE_PAE.bits() as u64;

		sregs.cr3 = BOOT_PML4;
		sregs.cr4 = cr4;
		sregs.cr0 = cr0;
		sregs.efer = EFER_LME | EFER_LMA;

		let mut seg = kvm_segment {
			base: 0,
			limit: 0xffffffff,
			selector: 1 << 3,
			present: 1,
			type_: 11,
			dpl: 0,
			db: 0,
			s: 1,
			l: 1,
			g: 1,
			..Default::default()
		};

		sregs.cs = seg;

		seg.type_ = 3;
		seg.selector = 2 << 3;
		seg.l = 0;
		sregs.ds = seg;
		sregs.es = seg;
		sregs.fs = seg;
		sregs.gs = seg;
		sregs.ss = seg;
		sregs.gdt.base = BOOT_GDT;
		sregs.gdt.limit = ((std::mem::size_of::<u64>() * BOOT_GDT_MAX as usize) - 1) as u16;

		self.vcpu.set_kvm_sregs(&sregs).unwrap();

		let mut regs = self.vcpu.get_kvm_regs().unwrap();
		regs.rflags = 2;
		regs.rip = entry_point;
		regs.rsp = 0x200000u64 - 0x1000u64;

		self.vcpu.set_kvm_regs(&regs).unwrap();
	}

	fn show_dtable(name: &str, dtable: &kvm_dtable) {
	    print!("{}                 {}\n", name, dtable);
	}

	fn show_segment(name: &str, seg: &kvm_segment) {
	    print!("{}       {}\n", name, seg);
	}
}

impl VirtualCPU for EhyveCPU {
	fn init(&mut self, entry_point: u64) -> Result<()>
	{
		self.setup_long_mode(entry_point);
		self.setup_cpuid();
		self.setup_msrs();

		Ok(())
	}

	fn run(&mut self) -> Result<()>
	{
		//self.print_registers();

		loop {
			self.vcpu.run().unwrap();
			let kvm_run = self.vcpu.kvm_run();
			match kvm_run.exit_reason {
				KVM_EXIT_HLT => {
					debug!("Halt Exit");
					break;
				},
				KVM_EXIT_IO => {
					let io = unsafe { &kvm_run.__bindgen_anon_1.io };

					if io.direction == KVM_EXIT_IO_OUT as u8 {
						if io.port == SHUTDOWN_PORT {
							return Ok(());
						} else {
							let data_addr = kvm_run as *const _ as u64 + io.data_offset;
							let data = unsafe { std::slice::from_raw_parts(data_addr as *const u8, io.size as usize) };

							self.io_exit(io.port, std::str::from_utf8(data).unwrap().to_string())?;
						}
					} else {
						info!("Unhandled IO exit: 0x{:x}", io.port);
					}
				},
				_ => {
					error!("Unknown exit reason: {:?}", kvm_run.exit_reason );
					//self.print_registers();

					return Err(Error::UnknownExitReason(kvm_run.exit_reason ));
				}
			}
		}

		Ok(())
	}

	fn print_registers(&self)
	{
		let regs = self.vcpu.get_kvm_regs().unwrap();
		let sregs = self.vcpu.get_kvm_sregs().unwrap();

		print!("\nDump state of CPU {}\n", self.id);
	    print!("\nRegisters:\n");
		print!("----------\n");
	    print!("{}{}", regs, sregs);

		print!("\nSegment registers:\n");
		print!("------------------\n");
		print!("register  selector  base              limit     type  p dpl db s l g avl\n");
	    EhyveCPU::show_segment("cs ", &sregs.cs);
	    EhyveCPU::show_segment("ss ", &sregs.ss);
	    EhyveCPU::show_segment("ds ", &sregs.ds);
	    EhyveCPU::show_segment("es ", &sregs.es);
	    EhyveCPU::show_segment("fs ", &sregs.fs);
	    EhyveCPU::show_segment("gs ", &sregs.gs);
	    EhyveCPU::show_segment("tr ", &sregs.tr);
	    EhyveCPU::show_segment("ldt", &sregs.ldt);
	    EhyveCPU::show_dtable("gdt", &sregs.gdt);
	    EhyveCPU::show_dtable("idt", &sregs.idt);

		print!("\nAPIC:\n");
		print!("-----\n");
		print!("efer: {:016x}  apic base: {:016x}\n", sregs.efer, sregs.apic_base);
	}
}

impl Drop for EhyveCPU {
    fn drop(&mut self) {
		debug!("Drop vCPU {}", self.id);
    }
}
