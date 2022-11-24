// Copyright (c) 2017-2021 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod kernel;
pub mod mm;

use crate::errno::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::write_bytes;
use goblin::elf::program_header::{PT_DYNAMIC, PT_GNU_RELRO, PT_LOAD};
use goblin::elf64::dynamic::{DT_RELA, DT_RELASZ};
use goblin::elf64::reloc::{R_386_GLOB_DAT, R_386_RELATIVE};
use goblin::{elf, elf64};
//use goblin::elf::header::{EM_X86_64,ET_EXEC};
//use goblin::elf64::section_header::{SHT_RELA,SHT_REL};
use self::mm::paging;
use self::mm::paging::{BasePageSize, PageSize, PageTableEntryFlags};
use self::mm::physicalmem;
use crate::consts::*;
use crate::fs;
use crate::logging::*;
use core::slice;
use x86::controlregs;

pub fn load_application(path: &String) -> Result<()> {
	unsafe {
		controlregs::cr3_write(paging::create_usr_pgd() as u64);
	}

	let mut file = fs::open(path, fs::OpenOptions::READONLY)?;
	let len = file.len();
	let mut buffer: Vec<u8> = Vec::new();

	buffer.resize(len, 0);
	file.read(&mut buffer)?;
	let elf = match elf::Elf::parse(&buffer) {
		Ok(n) => n,
		_ => return Err(Error::InvalidArgument),
	};
	debug!("elf information: {:#?}", &elf);

	if elf.is_lib == false || elf.is_64 == false {
		return Err(Error::InvalidArgument);
	}

	if elf.libraries.len() > 0 {
		error!(
			"Error: file depends on following libraries: {:?}",
			elf.libraries
		);
		return Err(Error::InvalidArgument);
	}

	// Determine the memory size of the executable
	let vstart: usize = 0;
	let mut exec_size: usize = 0;
	for i in &elf.program_headers {
		if i.p_type == PT_LOAD {
			/*if vstart == 0 {
				vstart = i.p_vaddr as usize;
			}*/

			exec_size = align_up!(
				i.p_vaddr as usize - vstart + i.p_memsz as usize,
				BasePageSize::SIZE
			);
		}
	}
	debug!("Virtual start address 0x{:x}", vstart);
	debug!("Memory size 0x{:x} {}", exec_size, len);

	if exec_size == 0 {
		error!("Error: unable to find PT_LOAD",);
		return Err(Error::InvalidArgument);
	}

	let physical_address = physicalmem::allocate(exec_size);
	paging::map::<BasePageSize>(
		USER_SPACE_START,
		physical_address,
		exec_size / BasePageSize::SIZE,
		PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE,
	);

	unsafe {
		write_bytes(USER_SPACE_START as *mut u8, 0x00, exec_size);
	}

	let mut rela_addr: u64 = 0;
	let mut relasz: u64 = 0;
	//let mut relaent: u64 = 0;
	for i in &elf.program_headers {
		if i.p_type == PT_LOAD {
			debug!("Load code for address 0x{:x}", i.p_vaddr);

			let mem = (USER_SPACE_START + i.p_vaddr as usize - vstart) as *mut u8;
			let mem_slice = unsafe { slice::from_raw_parts_mut(mem, i.p_filesz as usize) };

			mem_slice[0..i.p_filesz as usize].clone_from_slice(
				&buffer[(i.p_offset as usize)..(i.p_offset + i.p_filesz) as usize],
			);
		} else if i.p_type == PT_GNU_RELRO {
			debug!(
				"PT_GNU_RELRO at 0x{:x} (size 0x{:x})",
				i.p_vaddr, i.p_filesz
			);
		} else if i.p_type == PT_DYNAMIC {
			debug!("PT_DYNAMIC at 0x{:x} (size 0x{:x})", i.p_vaddr, i.p_filesz);

			let mem = (USER_SPACE_START + i.p_vaddr as usize - vstart) as *mut u8;
			let r#dyn = unsafe { elf::dynamic::dyn64::from_raw(0, mem as usize) };

			for j in r#dyn {
				if j.d_tag == DT_RELA {
					rela_addr = USER_SPACE_START as u64 + j.d_val;
				} else if j.d_tag == DT_RELASZ {
					relasz = j.d_val;
				} /*else if j.d_tag == DT_RELAENT {
					 relaent = j.d_val;
				 }*/
			}
		}
	}

	let rela = unsafe {
		elf64::reloc::from_raw_rela(rela_addr as *const elf64::reloc::Rela, relasz as usize)
	};
	for j in rela {
		let offset = (USER_SPACE_START - vstart + j.r_offset as usize) as *mut u64;

		if (j.r_info & 0xF) == R_386_RELATIVE as u64 {
			unsafe {
				*offset = (USER_SPACE_START as i64 - vstart as i64 + j.r_addend) as u64;
			}
		} else if (j.r_info & 0xF) == R_386_GLOB_DAT as u64 {
		} else {
			error!("Unsupported relocation type {}", j.r_info & 0xF);
		}
	}

	let entry = elf.entry - vstart as u64 + USER_SPACE_START as u64;

	debug!("jump to user land at 0x{:x}", entry);
	unsafe {
		self::kernel::jump_to_user_land(entry);
	}

	Ok(())
}
