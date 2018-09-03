use consts::*;
use vm::VirtualCPU;
use macos::error::*;
use hypervisor::{vCPU, x86Reg,read_vmx_cap};
use hypervisor::consts::vmcs::*;
use hypervisor::consts::vmx_cap::*;
use hypervisor;
use x86::bits64::segmentation::*;
use x86::shared::control_regs::*;
use x86::shared::msr::*;
use x86::shared::PrivilegeLevel;

const _EFER_LMA: u64	= 10; /* Long mode active (read-only) */
const EFER_LMA: u64		= (1 << _EFER_LMA);
const _EFER_LME: u64	= 8;  /* Long mode enable */
const EFER_LME: u64		= (1 << _EFER_LME);

#[derive(Debug)]
pub struct EhyveCPU
{
	id: u32,
	vcpu: vCPU
}

impl EhyveCPU {
    pub fn new(id: u32) -> EhyveCPU {
		EhyveCPU {
			id: id,
			vcpu: vCPU::new().unwrap()
		}
	}

	fn setup_system_gdt(&mut self) -> Result<()> {
		debug!("Setup GDT");

		self.vcpu.write_vmcs(VMCS_GUEST_CS_LIMIT, 0x000fffff).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_CS_BASE, 0).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_CS_AR, 	0xA09B).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_DS_LIMIT, 0x000fffff).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_DS_BASE, 0).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_DS_AR, 0xC093).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_ES_LIMIT, 0x000fffff).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_ES_BASE, 0).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_ES_AR, 0xC093).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_FS_LIMIT, 0x000fffff).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_FS_BASE, 0).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_FS_AR, 0xC093).or_else(to_error)?;

		//self.vcpu.write_vmcs(VMCS_GUEST_GDTR_BASE, 0).or_else(to_error)?;
		//self.vcpu.write_vmcs(VMCS_GUEST_GDTR_LIMIT, 0).or_else(to_error)?;
		//self.vcpu.write_vmcs(VMCS_GUEST_IDTR_BASE, 0).or_else(to_error)?;
		//self.vcpu.write_vmcs(VMCS_GUEST_IDTR_LIMIT, 0).or_else(to_error)?;

		// Reload the segment descriptors
		self.vcpu.write_vmcs(VMCS_GUEST_CS,
			SegmentSelector::new(GDT_KERNEL_CODE as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_DS,
			SegmentSelector::new(GDT_KERNEL_DATA as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_ES,
			SegmentSelector::new(GDT_KERNEL_DATA as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_SS,
			SegmentSelector::new(GDT_KERNEL_DATA as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_FS,
			SegmentSelector::new(GDT_KERNEL_DATA as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_GS,
			SegmentSelector::new(GDT_KERNEL_DATA as u16, PrivilegeLevel::Ring0).bits() as u64).or_else(to_error)?;

		Ok(())
	}

	fn setup_system_page_tables(&mut self) -> Result<()> {
		debug!("Setup page tables");
		self.vcpu.write_vmcs(VMCS_GUEST_CR3, 0x201000).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_CTRL_CR3_VALUE0, 0x201000).or_else(to_error)?;

		Ok(())
	}

	fn setup_system_64bit(&mut self) -> Result<()> {
		debug!("Setup 64bit mode");

		/*self.vcpu.write_vmcs(VMCS_CTRL_CR0_MASK, !0u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_CTRL_CR4_MASK, !0u64).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_CTRL_CR4_SHADOW, 0).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_CTRL_CR0_SHADOW, 0).or_else(to_error)?;*/

		let value = CR0_PROTECTED_MODE | CR0_ENABLE_PAGING | CR0_CACHE_DISABLE |
					CR0_NOT_WRITE_THROUGH | CR0_EXTENSION_TYPE;
		self.vcpu.write_vmcs(VMCS_GUEST_CR0, value.bits() as u64).or_else(to_error)?;

		let value = CR4_ENABLE_PAE | CR4_ENABLE_PPMC;
		self.vcpu.write_vmcs(VMCS_GUEST_CR4, value.bits() as u64).or_else(to_error)?;

		let value = EFER_LME | EFER_LMA;
		self.vcpu.write_vmcs(VMCS_GUEST_IA32_EFER, value).or_else(to_error)?;

		Ok(())
	}

	fn setup_msr(&mut self) -> Result<()> {
		debug!("Enable MSR registers");

		//self.vcpu.enable_native_msr(IA32_EFER, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_FS_BASE, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_GS_BASE, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_KERNEL_GSBASE, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_SYSENTER_CS, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_SYSENTER_EIP, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_SYSENTER_ESP, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_STAR, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_LSTAR, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_CSTAR, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_FMASK, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(TSC, true).or_else(to_error)?;
		self.vcpu.enable_native_msr(IA32_TSC_AUX, true).or_else(to_error)?;

		Ok(())
	}

	/* desired control word constrained by hardware/hypervisor capabilities */
	fn cap2ctrl(cap: u64, ctrl: u64) -> u64
	{
		((ctrl | (cap & 0xffffffffu64)) & (cap >> 32))
	}
}

impl VirtualCPU for EhyveCPU {
	fn init(&mut self, entry_point: u64) -> Result<()>
	{
		/* read hypervisor enforced capabilities of the machine, (see Intel docs) */
		/*let mut vmx_cap_pinbased = read_vmx_cap(&hypervisor::VMXCap::PINBASED).unwrap();
		let mut vmx_cap_procbased = read_vmx_cap(&hypervisor::VMXCap::PROCBASED).unwrap();
		let mut vmx_cap_procbased2 = read_vmx_cap(&hypervisor::VMXCap::PROCBASED2).unwrap();
		let mut vmx_cap_entry = read_vmx_cap(&hypervisor::VMXCap::ENTRY).unwrap();
		let mut vmx_cap_exit = read_vmx_cap(&hypervisor::VMXCap::EXIT).unwrap();*/

		self.setup_msr()?;

		/*vmx_cap_pinbased = vmx_cap_pinbased | PIN_BASED_INTR | PIN_BASED_NMI | PIN_BASED_VIRTUAL_NMI;
		vmx_cap_pinbased = vmx_cap_pinbased & !PIN_BASED_PREEMPTION_TIMER;
		self.vcpu.write_vmcs(VMCS_CTRL_PIN_BASED, vmx_cap_pinbased).or_else(to_error)?;

		vmx_cap_procbased = vmx_cap_procbased | CPU_BASED_SECONDARY_CTLS | CPU_BASED_MONITOR | CPU_BASED_MWAIT;
		vmx_cap_procbased = vmx_cap_procbased | CPU_BASED_CR8_STORE | CPU_BASED_CR8_LOAD | CPU_BASED_HLT;
		self.vcpu.write_vmcs(VMCS_CTRL_CPU_BASED, vmx_cap_procbased).or_else(to_error)?;

		vmx_cap_procbased2 = vmx_cap_procbased2 | CPU_BASED2_RDTSCP;
		self.vcpu.write_vmcs(VMCS_CTRL_CPU_BASED2, vmx_cap_procbased2).or_else(to_error)?;

		vmx_cap_entry = vmx_cap_entry | VMENTRY_LOAD_EFER;
		self.vcpu.write_vmcs(VMCS_CTRL_VMEXIT_CONTROLS, vmx_cap_entry).or_else(to_error)?;

		vmx_cap_exit = vmx_cap_exit | VMEXIT_HOST_IA32E|VMEXIT_LOAD_EFER;
		self.vcpu.write_vmcs(VMCS_CTRL_VMENTRY_CONTROLS, vmx_cap_exit).or_else(to_error)?;

		self.vcpu.write_vmcs(VMCS_CTRL_EXC_BITMAP, 0xffffffffu64).or_else(to_error)?;

		vmx_cap_pinbased = read_vmx_cap(&hypervisor::VMXCap::PINBASED).unwrap();
		debug!("VMX Pinbased 0x{:x}", vmx_cap_pinbased);
		vmx_cap_procbased = read_vmx_cap(&hypervisor::VMXCap::PROCBASED).unwrap();
		debug!("VMX Procbased 0x{:x}", vmx_cap_procbased);
		vmx_cap_procbased2 = read_vmx_cap(&hypervisor::VMXCap::PROCBASED2).unwrap();
		debug!("VMX Procbased2 0x{:x}", vmx_cap_procbased2);
		vmx_cap_entry = read_vmx_cap(&hypervisor::VMXCap::ENTRY).unwrap();
		debug!("VMX Entry 0x{:x}", vmx_cap_entry);
		vmx_cap_exit = read_vmx_cap(&hypervisor::VMXCap::EXIT).unwrap();
		debug!("VMX Exit 0x{:x}", vmx_cap_exit);*/

		debug!("Setup APIC");
		self.vcpu.set_apic_addr(APIC_DEFAULT_BASE).or_else(to_error)?;

		debug!("Setup instruction pointers");
		self.vcpu.write_vmcs(VMCS_GUEST_RIP, entry_point).or_else(to_error)?;
		self.vcpu.write_vmcs(VMCS_GUEST_RFLAGS, 0x2).or_else(to_error)?;

		self.setup_system_gdt()?;
        self.setup_system_page_tables()?;
		self.setup_system_64bit()?;

		Ok(())
	}

	fn run(&mut self) -> Result<()>
	{
		debug!("Run vCPU {}", self.id);
		loop {
			self.vcpu.run().or_else(to_error)?;

			let reason = self.vcpu.read_vmcs(VMCS_RO_EXIT_REASON).unwrap() & 0xffff;
			match reason {
				/*VMX_REASON_VMPTRLD => {
					info!("Handle VMX_REASON_VMPTRLD");
					self.print_registers();
				},*/
				_ => {
					error!("Unhandled exit: 0x{:x}", reason);
					self.print_registers();
					return Err(Error::UnhandledExitReason);
				}
			};
		}

		//Ok(())
	}

	fn print_registers(&self)
	{
		println!("CPU Id : {}", self.id);

		let value1 = self.vcpu.read_register(&x86Reg::RIP).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::RFLAGS).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "RIP", value1, "RFLAGS", value2);
		let value1 = self.vcpu.read_register(&x86Reg::RAX).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::RBX).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "RAX", value1, "RBX", value2);
		let value1 = self.vcpu.read_register(&x86Reg::RCX).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::RDX).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "RCX", value1, "RDX", value2);
		let value1 = self.vcpu.read_register(&x86Reg::RSI).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::RDI).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "RSI", value1, "RDI", value2);
		let value1 = self.vcpu.read_register(&x86Reg::R8).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::R9).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "R8", value1, "R9", value2);
		let value1 = self.vcpu.read_register(&x86Reg::R10).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::R11).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "R10", value1, "R11", value2);
		let value1 = self.vcpu.read_register(&x86Reg::R12).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::R13).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}", "R12", value1, "R13", value2);
		let value1 = self.vcpu.read_register(&x86Reg::R14).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::R15).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "R14", value1, "R15", value2);
		let value1 = self.vcpu.read_register(&x86Reg::CS).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::DS).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "CS", value1, "DS", value2);
		let value1 = self.vcpu.read_register(&x86Reg::ES).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::SS).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "ES", value1, "SS", value2);
		let value1 = self.vcpu.read_register(&x86Reg::FS).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::GS).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "FS", value1, "GS", value2);
		let value1 = self.vcpu.read_register(&x86Reg::CR0).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::CR1).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "CR0", value1, "CR1", value2);
		let value1 = self.vcpu.read_register(&x86Reg::CR2).unwrap();
		let value2 = self.vcpu.read_register(&x86Reg::CR3).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "CR2", value1, "CR3", value2);
		let value1 = self.vcpu.read_register(&x86Reg::CR4).unwrap();
		let value2 = self.vcpu.read_vmcs(VMCS_GUEST_IA32_EFER).unwrap();
		println!("{0: <7}: 0x{1:0>16x}   {2: <7}: 0x{3:0>16x}	", "CR4", value1, "EFER", value2);
	}
}

impl Drop for EhyveCPU {
    fn drop(&mut self) {
        debug!("Drop virtual CPU {}", self.id);
		let _ = self.vcpu.destroy();
    }
}
