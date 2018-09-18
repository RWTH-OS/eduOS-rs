use std;
use std::mem;
use std::rc::Rc;
use libc::c_void;
use vm::{Vm, VirtualCPU};
use consts::*;
use error::*;
use windows::vcpu::*;
use aligned_alloc::*;
use libwhp::*;
use libwhp::memory::*;

fn check_hypervisor() {
    let capability =
        get_capability(WHV_CAPABILITY_CODE::WHvCapabilityCodeHypervisorPresent).unwrap();
    if unsafe { capability.HypervisorPresent } == FALSE {
        panic!("Hypervisor not present");
    }
}

pub struct Ehyve {
	entry_point: u64,
	mem_size: usize,
	partition: Rc<Partition>,
	guest_mem: Rc<VirtualMemory>,
    gpa_mapping : Rc<GPARangeMapping>,
	num_cpus: u32,
	path: String
}

impl Ehyve {
    pub fn new(path: String, mem_size: usize, num_cpus: u32) -> Result<Ehyve> {
		check_hypervisor();

		let mut p = Partition::new().unwrap();
	    Ehyve::setup_partition(&mut p);

	    let payload_mem = VirtualMemory::new(mem_size).unwrap();

	    let guest_address: WHV_GUEST_PHYSICAL_ADDRESS = 0;

	    let mapping = p.map_gpa_range(
	        &payload_mem,
	        guest_address,
	        payload_mem.get_size() as UINT64,
	        WHV_MAP_GPA_RANGE_FLAGS::WHvMapGpaRangeFlagRead
	            | WHV_MAP_GPA_RANGE_FLAGS::WHvMapGpaRangeFlagWrite
	            | WHV_MAP_GPA_RANGE_FLAGS::WHvMapGpaRangeFlagExecute,
	    ).unwrap();

		let hyve = Ehyve {
			entry_point: 0,
			mem_size: mem_size,
			partition: Rc::new(p),
			guest_mem: Rc::new(payload_mem),
            gpa_mapping: Rc::new(mapping),
			num_cpus: num_cpus,
			path: path
        };

        hyve.init_guest_mem();

		Ok(hyve)
	}

	fn setup_partition(partition: &mut Partition) {
	    let mut property: WHV_PARTITION_PROPERTY = unsafe { std::mem::zeroed() };
	    property.ProcessorCount = 1;
	    partition.set_property(
	        WHV_PARTITION_PROPERTY_CODE::WHvPartitionPropertyCodeProcessorCount,
	        &property,
	    ).unwrap();

	    property = unsafe { std::mem::zeroed() };
	    unsafe {
	        property.ExtendedVmExits.set_X64CpuidExit(1);
	        property.ExtendedVmExits.set_X64MsrExit(1);
	        property.ExtendedVmExits.set_ExceptionExit(1);
	    }

	    partition.set_property(
	        WHV_PARTITION_PROPERTY_CODE::WHvPartitionPropertyCodeExtendedVmExits,
	        &property,
	    ).unwrap();

	    let cpuids: [UINT32; 1] = [1];
	    partition.set_property_cpuid_exits(&cpuids).unwrap();

	    let mut cpuid_results: [WHV_X64_CPUID_RESULT; 1] = unsafe { std::mem::zeroed() };

	    cpuid_results[0].Function = 0x40000000;
	    let mut id_reg_values: [UINT32; 3] = [0; 3];
	    let id = "libwhp\0";
	    unsafe {
	        std::ptr::copy_nonoverlapping(id.as_ptr(), id_reg_values.as_mut_ptr() as *mut u8, id.len());
	    }
	    cpuid_results[0].Ebx = id_reg_values[0];
	    cpuid_results[0].Ecx = id_reg_values[1];
	    cpuid_results[0].Edx = id_reg_values[2];

	    partition.set_property_cpuid_results(&cpuid_results).unwrap();

	    partition.setup().unwrap();
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
		(self.guest_mem.as_ptr() as *mut u8, self.mem_size)
	}

	fn kernel_path(&self) -> &str {
			&self.path
	}

	fn create_cpu(&self, id: u32) -> Result<Box<VirtualCPU>> {
        let vcpu = self.partition.create_virtual_processor(id).unwrap();
		Ok(Box::new(EhyveCPU::new(id, vcpu)))
	}
}

impl Drop for Ehyve {
    fn drop(&mut self) {
        debug!("Drop virtual machine");

		//unmap_mem(0, self.mem_size).unwrap();

		//unsafe { aligned_free(self.guest_mem); }
    }
}

unsafe impl Send for Ehyve {}
unsafe impl Sync for Ehyve {}
