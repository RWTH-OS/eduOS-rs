pub mod error;
mod utils;
mod proto;
mod gdt;
mod kvm;
mod vcpu;
mod vm;

pub mod ehyve;

// reexport Uhyve to show up in the root namespace of our module
pub use self::ehyve::*;

use std::env;

use linux::error::*;

#[derive(Debug, Clone)]
pub enum VmParameter {
    Kvm {
        mem_size: u64,
        num_cpus: u32,
    }
}

impl VmParameter {
    pub fn parse_bool(name: &str, default: bool) -> bool {
        env::var(name).map(|x| x.parse::<i32>().unwrap_or(default as i32) != 0).unwrap_or(default)
    }

    pub fn from_env() -> VmParameter {
        let mem_size: u64 = env::var("EHYVE_MEM").map(|x| utils::parse_mem(&x).unwrap_or(512*1024*1024)).unwrap_or(512*1024*1024);
        let num_cpus: u32 = env::var("EHYVE_CPUS").map(|x| x.parse().unwrap_or(1)).map(|x| if x == 0 { 1 } else { x }).unwrap_or(1);


        VmParameter::Kvm {
            mem_size: mem_size,
            num_cpus: num_cpus,
        }
    }
}

pub trait Vm {
    fn num(&self) -> u8;
    fn run(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

/*use linux::VmParameter;
use linux::Vm;
use linux::ehyve::Ehyve;
use linux::error::Result;*/

pub fn create_vm(path: Option<String>, specs: VmParameter) -> Result<()> {
    let mut vm: Box<Vm> = match specs {
        VmParameter::Kvm{ mem_size, num_cpus } => Box::new(Ehyve::new(path, mem_size, num_cpus)?)
    };

    vm.run()?;

    Ok(())
}
