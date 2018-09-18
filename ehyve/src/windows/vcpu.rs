use std;
use consts::*;
use vm::VirtualCPU;
use error::*;
use x86::bits64::segmentation::*;
use x86::shared::control_regs::*;
use x86::shared::msr::*;
use x86::shared::PrivilegeLevel;
use libwhp::instruction_emulator::*;
use libwhp::memory::*;
use libwhp::*;

pub struct EhyveCPU
{
	id: u32,
	vcpu: VirtualProcessor
}

impl EhyveCPU {
    pub fn new(id: u32, vcpu: VirtualProcessor) -> EhyveCPU {
		EhyveCPU {
			id: id,
			vcpu: vcpu
		}
	}
}

impl VirtualCPU for EhyveCPU {
	fn init(&mut self, entry_point: u64) -> Result<()>
	{
	    const NUM_REGS: UINT32 = 13;
	    let mut reg_names: [WHV_REGISTER_NAME; NUM_REGS as usize] = unsafe { std::mem::zeroed() };
	    let mut reg_values: [WHV_REGISTER_VALUE; NUM_REGS as usize] = unsafe { std::mem::zeroed() };

	    // Setup paging
	    reg_names[0] = WHV_REGISTER_NAME::WHvX64RegisterCr3;
	    reg_values[0].Reg64 = BOOT_PML4;
	    reg_names[1] = WHV_REGISTER_NAME::WHvX64RegisterCr4;
	    reg_values[1].Reg64 = CR4_ENABLE_PAE.bits() as u64;
	    reg_names[2] = WHV_REGISTER_NAME::WHvX64RegisterCr0;
	    reg_values[2].Reg64 = (CR0_PROTECTED_MODE | CR0_ENABLE_PAGING | CR0_EXTENSION_TYPE | CR0_NUMERIC_ERROR).bits() as u64;
	    reg_names[3] = WHV_REGISTER_NAME::WHvX64RegisterEfer;
	    reg_values[3].Reg64 = EFER_LME | EFER_LMA;

	    reg_names[4] = WHV_REGISTER_NAME::WHvX64RegisterCs;
	    unsafe {
	        let segment = &mut reg_values[4].Segment;
	        segment.Base = 0;
	        segment.Limit = 0xffffffff;
	        segment.Selector = 1 << 3;
	        segment.set_SegmentType(11);
	        segment.set_NonSystemSegment(1);
	        segment.set_Present(1);
	        segment.set_Long(1);
			segment.set_Default(0);
	        segment.set_Granularity(1);
	    }

	    reg_names[5] = WHV_REGISTER_NAME::WHvX64RegisterDs;
	    unsafe {
	        let segment = &mut reg_values[5].Segment;
	        segment.Base = 0;
	        segment.Limit = 0xffffffff;
	        segment.Selector = 2 << 3;
	        segment.set_SegmentType(3);
	        segment.set_NonSystemSegment(1);
	        segment.set_Present(1);
	        segment.set_Long(0);
			segment.set_Default(1);
	        segment.set_Granularity(1);
	    }

	    reg_names[6] = WHV_REGISTER_NAME::WHvX64RegisterEs;
	    reg_values[6] = reg_values[5];

	    reg_names[7] = WHV_REGISTER_NAME::WHvX64RegisterFs;
	    reg_values[7] = reg_values[5];

	    reg_names[8] = WHV_REGISTER_NAME::WHvX64RegisterGs;
	    reg_values[8] = reg_values[5];

	    reg_names[9] = WHV_REGISTER_NAME::WHvX64RegisterSs;
	    reg_values[9] = reg_values[5];

	    reg_names[10] = WHV_REGISTER_NAME::WHvX64RegisterRflags;
	    reg_values[10].Reg64 = 0x2;
	    reg_names[11] = WHV_REGISTER_NAME::WHvX64RegisterRip;
	    reg_values[11].Reg64 = entry_point;
	    // Create stack
	    reg_names[12] = WHV_REGISTER_NAME::WHvX64RegisterRsp;
	    reg_values[12].Reg64 = 0x200000 - 0x1000;

	    self.vcpu.set_registers(&reg_names, &reg_values).unwrap();

		Ok(())
	}

	fn run(&mut self) -> Result<()>
	{
		debug!("Run vCPU {}", self.id);
		loop {
			let exit_context = self.vcpu.run().unwrap();

			match exit_context.ExitReason {
				WHV_RUN_VP_EXIT_REASON::WHvRunVpExitReasonX64IoPortAccess => {
					let mut e = Emulator::new(self).unwrap();
					let io_port_access_ctx = unsafe { &exit_context.anon_union.IoPortAccess };

					if io_port_access_ctx.PortNumber == SHUTDOWN_PORT {
						return Ok(());
					}

					let _status = e.try_io_emulation(
				        std::ptr::null_mut(),
				        &exit_context.VpContext,
				        io_port_access_ctx,
				    ).unwrap();
				},
				_ => {
					error!("Unhandled exit reason: {:?}", exit_context.ExitReason);
					self.print_registers();
					return Err(Error::UnhandledExitReason);
				}
			}
		}
	}

	fn print_registers(&self)
	{
		print!("\nDump state of CPU {}\n", self.id);
		print!("\nRegisters:\n");
		print!("----------\n");

		const NUM_REGS: UINT32 = 34;
	    let mut reg_names: [WHV_REGISTER_NAME; NUM_REGS as usize] = unsafe { std::mem::zeroed() };
	    let mut reg_values: [WHV_REGISTER_VALUE; NUM_REGS as usize] = unsafe { std::mem::zeroed() };

		reg_names[0] = WHV_REGISTER_NAME::WHvX64RegisterRip;
		reg_names[1] = WHV_REGISTER_NAME::WHvX64RegisterRsp;
		reg_names[2] = WHV_REGISTER_NAME::WHvX64RegisterRflags;
		reg_names[3] = WHV_REGISTER_NAME::WHvX64RegisterRax;
		reg_names[4] = WHV_REGISTER_NAME::WHvX64RegisterRbx;
		reg_names[5] = WHV_REGISTER_NAME::WHvX64RegisterRcx;
		reg_names[6] = WHV_REGISTER_NAME::WHvX64RegisterRdx;
		reg_names[7] = WHV_REGISTER_NAME::WHvX64RegisterRsi;
		reg_names[8] = WHV_REGISTER_NAME::WHvX64RegisterRdi;
		reg_names[9] = WHV_REGISTER_NAME::WHvX64RegisterRbp;
		reg_names[10] = WHV_REGISTER_NAME::WHvX64RegisterR8;
		reg_names[11] = WHV_REGISTER_NAME::WHvX64RegisterR9;
		reg_names[12] = WHV_REGISTER_NAME::WHvX64RegisterR10;
		reg_names[13] = WHV_REGISTER_NAME::WHvX64RegisterR11;
		reg_names[14] = WHV_REGISTER_NAME::WHvX64RegisterR12;
		reg_names[15] = WHV_REGISTER_NAME::WHvX64RegisterR13;
		reg_names[16] = WHV_REGISTER_NAME::WHvX64RegisterR14;
		reg_names[17] = WHV_REGISTER_NAME::WHvX64RegisterR15;
		reg_names[18] = WHV_REGISTER_NAME::WHvX64RegisterCr0;
		reg_names[19] = WHV_REGISTER_NAME::WHvX64RegisterCr2;
		reg_names[20] = WHV_REGISTER_NAME::WHvX64RegisterCr3;
		reg_names[21] = WHV_REGISTER_NAME::WHvX64RegisterCr4;
		reg_names[22] = WHV_REGISTER_NAME::WHvX64RegisterCs;
		reg_names[23] = WHV_REGISTER_NAME::WHvX64RegisterDs;
		reg_names[24] = WHV_REGISTER_NAME::WHvX64RegisterEs;
		reg_names[25] = WHV_REGISTER_NAME::WHvX64RegisterSs;
		reg_names[26] = WHV_REGISTER_NAME::WHvX64RegisterFs;
		reg_names[27] = WHV_REGISTER_NAME::WHvX64RegisterGs;
		reg_names[28] = WHV_REGISTER_NAME::WHvX64RegisterTr;
		reg_names[29] = WHV_REGISTER_NAME::WHvX64RegisterLdtr;
		reg_names[30] = WHV_REGISTER_NAME::WHvX64RegisterGdtr;
		reg_names[31] = WHV_REGISTER_NAME::WHvX64RegisterIdtr;
		reg_names[32] = WHV_REGISTER_NAME::WHvX64RegisterEfer;
		reg_names[33] = WHV_REGISTER_NAME::WHvX64RegisterApicBase;

		self.vcpu.get_registers(&reg_names, &mut reg_values).unwrap();

		unsafe {
			print!("rip: {:016x}   rsp: {:016x} flags: {:016x}\n\
				rax: {:016x}   rbx: {:016x}   rcx: {:016x}\n\
				rdx: {:016x}   rsi: {:016x}   rdi: {:016x}\n\
				rbp: {:016x}    r8: {:016x}    r9: {:016x}\n\
				r10: {:016x}   r11: {:016x}   r12: {:016x}\n\
				r13: {:016x}   r14: {:016x}   r15: {:016x}\n",
				reg_values[0].Reg64, reg_values[1].Reg64, reg_values[2].Reg64,
				reg_values[3].Reg64, reg_values[4].Reg64, reg_values[5].Reg64,
				reg_values[6].Reg64, reg_values[7].Reg64, reg_values[8].Reg64,
				reg_values[9].Reg64, reg_values[10].Reg64, reg_values[11].Reg64,
				reg_values[12].Reg64, reg_values[13].Reg64, reg_values[14].Reg64,
				reg_values[15].Reg64, reg_values[16].Reg64, reg_values[17].Reg64);

			print!("cr0: {:016x}   cr2: {:016x}   cr3: {:016x}\ncr4: {:016x}\n",
				reg_values[18].Reg64, reg_values[19].Reg64, reg_values[20].Reg64, reg_values[21].Reg64);

			print!("\nSegment registers:\n");
			print!("------------------\n");
			print!("register  selector  base              limit     type  p dpl db s l g avl\n");

			let segment = &reg_values[22].Segment;
			println!("cs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[23].Segment;
			println!("ds        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
					segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
					segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[24].Segment;
			println!("es        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[25].Segment;
			println!("ss        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[26].Segment;
			println!("fs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[27].Segment;
			println!("gs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[28].Segment;
			println!("tr        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let segment = &reg_values[29].Segment;
			println!("ldtr      {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
				segment.Selector, segment.Base, segment.Limit, segment.SegmentType(), segment.Present(), segment.DescriptorPrivilegeLevel(), segment.Default(),
				segment.NonSystemSegment(), segment.Long(), segment.Granularity(), segment.Available());
			let table = &reg_values[30].Table;
			println!("gdt                 {:016x}  {:08x}", table.Base, table.Limit);
			let table = &reg_values[31].Table;
			println!("idt                 {:016x}  {:08x}", table.Base, table.Limit);

			println!("\nAPIC:");
			println!("-----");
			println!("efer: 0x{:016x}  apic base: 0x{:016x}", reg_values[32].Reg64, reg_values[33].Reg64);
		}
	}
}

impl Drop for EhyveCPU {
    fn drop(&mut self) {
        debug!("Drop virtual CPU {}", self.id);
	}
}

impl EmulatorCallbacks for EhyveCPU {
    fn io_port(
        &mut self,
        _context: *mut VOID,
        io_access: &mut WHV_EMULATOR_IO_ACCESS_INFO,
    ) -> HRESULT {
		let cstr = unsafe {
			std::str::from_utf8(std::slice::from_raw_parts(&io_access.Data as *const _ as *const u8,
				io_access.AccessSize as usize)).unwrap()
		};

		self.io_exit(io_access.Port, cstr.to_string()).unwrap();

        S_OK
    }

    fn memory(
        &mut self,
        _context: *mut VOID,
        _memory_access: &mut WHV_EMULATOR_MEMORY_ACCESS_INFO,
    ) -> HRESULT {
        /*match memory_access.AccessSize {
            8 => match memory_access.Direction {
                0 => {
                    let data = &memory_access.Data as *const _ as *mut u64;
                    unsafe {
                        *data = 0x1000;
                        println!("MMIO read: 0x{:x}", *data);
                    }
                }
                _ => {
                    let value = unsafe { *(&memory_access.Data as *const _ as *const u64) };
                    println!("MMIO write: 0x{:x}", value);
                }
            },
            _ => println!("Unsupported MMIO access size: {}", memory_access.AccessSize),
        }*/
		panic!("memory() ist currently unsupported");

        S_OK
    }

    fn get_virtual_processor_registers(
        &mut self,
        _context: *mut VOID,
        register_names: &[WHV_REGISTER_NAME],
        register_values: &mut [WHV_REGISTER_VALUE],
    ) -> HRESULT {
        self.vcpu.get_registers(register_names, register_values).unwrap();

        S_OK
    }

    fn set_virtual_processor_registers(
        &mut self,
        _context: *mut VOID,
        register_names: &[WHV_REGISTER_NAME],
        register_values: &[WHV_REGISTER_VALUE],
    ) -> HRESULT {
        self.vcpu.set_registers(register_names, register_values).unwrap();

        S_OK
    }

    fn translate_gva_page(
        &mut self,
        _context: *mut VOID,
        gva: WHV_GUEST_VIRTUAL_ADDRESS,
        translate_flags: WHV_TRANSLATE_GVA_FLAGS,
        translation_result: &mut WHV_TRANSLATE_GVA_RESULT_CODE,
        gpa: &mut WHV_GUEST_PHYSICAL_ADDRESS,
    ) -> HRESULT {
        /*let (translation_result1, gpa1) = self.vp_ref_cell
            .borrow()
            .translate_gva(gva, translate_flags)
            .unwrap();
        *translation_result = translation_result1.ResultCode;
        *gpa = gpa1;*/
		panic!("translate_gva_page() is currently unsupported");

        S_OK
    }
}
