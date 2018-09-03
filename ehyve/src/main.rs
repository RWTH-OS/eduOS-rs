#![feature(untagged_unions)]
#![feature(core_intrinsics)]
#![allow(dead_code)]

extern crate libc;
extern crate memmap;
extern crate elf;
extern crate x86;
extern crate raw_cpuid;
extern crate aligned_alloc;
#[cfg(target_os = "macos")]
extern crate hypervisor;

#[macro_use]
extern crate log;
extern crate env_logger;

mod vm;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
pub mod utils;
pub mod consts;

use std::env;
use std::thread;
use std::sync::Arc;
use vm::*;

fn main() {
	env_logger::init();

    let path = env::args().nth(1).expect("Expect path to the kernel!");
	let mut vm = create_vm(path, VmParameter::from_env()).unwrap();
	let num_cpus = vm.num_cpus();

	vm.load_kernel().unwrap();

	let vm = Arc::new(vm);
	let threads: Vec<_> = (0..num_cpus)
		.map(|tid| {
			let vm = vm.clone();

			thread::spawn(move || {
				debug!("Create thread for CPU {}", tid);

				let mut cpu = vm.create_cpu(tid).unwrap();
				cpu.init(vm.get_entry_point()).unwrap();

				cpu.run().unwrap();
			})
		}).collect();

	for t in threads {
		t.join().unwrap();
	}
}
