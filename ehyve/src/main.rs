#![feature(untagged_unions)]
#![feature(core_intrinsics)]
#![feature(unique)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate libc;
extern crate memmap;
extern crate elf;
extern crate inotify;
extern crate byteorder;
extern crate raw_cpuid;
extern crate rand;

#[macro_use]
extern crate chan;
extern crate chan_signal;

#[macro_use]
extern crate nix;

#[macro_use]
extern crate log;
extern crate env_logger;

mod vm;

use std::env;
use std::process;

use vm::VmParameter;
use vm::Vm;
use vm::ehyve::Ehyve;
use vm::error::Result;

fn create_vm(path: Option<String>, specs: VmParameter) -> Result<()> {
    let mut vm: Box<Vm> = match specs {
        VmParameter::Kvm{ mem_size, num_cpus } => Box::new(Ehyve::new(path, mem_size, num_cpus)?)
    };

    vm.run()?;

    Ok(())
}

fn main() {
	    env_logger::init();
    let verbose = VmParameter::parse_bool("EHYVE_VERBOSE", false);
    unsafe { vm::VERBOSE = verbose; }

    let path = env::args().nth(1);
    if let Err(e) = create_vm(path, VmParameter::from_env()) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
