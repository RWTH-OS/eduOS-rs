#![feature(untagged_unions)]
#![feature(core_intrinsics)]
#![allow(dead_code)]

extern crate libc;
extern crate memmap;
extern crate elf;
extern crate x86;
extern crate raw_cpuid;
extern crate aligned_alloc;
#[macro_use]
extern crate lazy_static;
#[cfg(target_os = "macos")]
extern crate hypervisor;
#[cfg(target_os = "windows")]
extern crate libwhp;
#[cfg(target_os = "linux")]
extern crate libkvm;
#[cfg(target_os = "windows")]
extern crate kernel32;

#[macro_use]
extern crate log;
extern crate env_logger;

mod vm;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
pub mod utils;
pub mod consts;
pub mod error;

use std::env;
use std::thread;
use std::sync::Arc;
use vm::*;
use error::*;

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

				let result = cpu.run();
				match result {
					Ok(()) => {},
					Err(Error::Shutdown) => {
						info!("Receive shutdown command!");
					},
					_ => {
						error!("CPU {} crashes!", tid);
					}
				}
			})
		}).collect();

	for t in threads {
		t.join().unwrap();
	}
}
