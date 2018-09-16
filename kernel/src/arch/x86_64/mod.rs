pub mod serial;
pub mod processor;

#[repr(C)]
pub struct KernelHeader {
	num_cpus: u32,
	cpus_online: u32,
	cpu_freq: u32,
	mem_limit: usize
}

#[link_section = ".kheader"]
pub static KERNEL_HEADER: KernelHeader = KernelHeader {
	num_cpus: 1,
	cpus_online: 0,
	cpu_freq: 0,
	mem_limit: 0
};

pub fn get_cpufreq() -> u32
{
	KERNEL_HEADER.cpu_freq
}
