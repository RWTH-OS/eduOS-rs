pub mod kernel;
pub mod mm;

use self::mm::paging;
use self::mm::paging::{BasePageSize, PageSize, PageTableEntryFlags};
use self::mm::physicalmem;
use crate::consts::*;
use crate::fs;
use crate::io::{self, Read};
use crate::logging::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::write_bytes;
use core::slice;
use goblin::elf::program_header::{PT_DYNAMIC, PT_GNU_RELRO, PT_LOAD};
use goblin::elf64::dynamic::{DT_RELA, DT_RELASZ};
use goblin::elf64::reloc::{R_386_GLOB_DAT, R_386_RELATIVE};
use goblin::{elf, elf64};
use x86::controlregs;

pub fn load_application(path: &String) -> io::Result<()> {
	debug!("Try to load application!");
	unsafe {
		controlregs::cr3_write(paging::create_usr_pgd().as_u64());
	}

	let mut file = fs::File::open(path)?;
	let len = file.len()?;
	let mut buffer: Vec<u8> = Vec::new();

	buffer.resize(len, 0);
	file.read(&mut buffer)?;
	let elf = match elf::Elf::parse(&buffer) {
		Ok(n) => n,
		_ => return Err(io::Error::EINVAL),
	};
	drop(file); // close file
	debug!("elf information: {:#?}", &elf);

    if !elf.is_lib {
        info!("File is an ELF executable");
    } else {
        return Err(io::Error::EINVAL);
    }

	if elf.is_64 {
        info!("File is a 64bit ELF executable");
    } else {
		return Err(io::Error::EINVAL);
	}

	if elf.libraries.len() > 0 {
		error!(
			"Error: file depends on following libraries: {:?}",
			elf.libraries
		);
		return Err(io::Error::EINVAL);
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
		return Err(io::Error::EINVAL);
	}

	let physical_address = physicalmem::allocate(exec_size);
	paging::map::<BasePageSize>(
		USER_ENTRY,
		physical_address,
		exec_size / BasePageSize::SIZE,
		PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE,
	);

	unsafe {
		write_bytes(USER_ENTRY.as_mut_ptr() as *mut u8, 0x00, exec_size);
	}

	let mut rela_addr: u64 = 0;
	let mut relasz: u64 = 0;
	//let mut relaent: u64 = 0;
	for i in &elf.program_headers {
		if i.p_type == PT_LOAD {
			debug!("Load code for address 0x{:x}", i.p_vaddr);

			let mem = (USER_ENTRY.as_usize() + i.p_vaddr as usize - vstart) as *mut u8;
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

			let mem = (USER_ENTRY.as_u64() + i.p_vaddr as u64 - vstart as u64) as *mut u8;
			let r#dyn = unsafe { elf::dynamic::dyn64::from_raw(0, mem as usize) };

			for j in r#dyn {
				if j.d_tag == DT_RELA {
					rela_addr = USER_ENTRY.as_u64() + j.d_val;
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
		let offset = (USER_ENTRY.as_usize() - vstart + j.r_offset as usize) as *mut u64;

		if (j.r_info & 0xF) == R_386_RELATIVE as u64 {
			unsafe {
				*offset = (USER_ENTRY.as_usize() as i64 - vstart as i64 + j.r_addend) as u64;
			}
		} else if (j.r_info & 0xF) == R_386_GLOB_DAT as u64 {
		} else {
			error!("Unsupported relocation type {}", j.r_info & 0xF);
		}
	}

	let entry = elf.entry as usize - vstart as usize + USER_ENTRY.as_usize();

	// free temporary buffer
	drop(buffer);

	debug!("jump to user land at 0x{:x}", entry);
	unsafe {
		self::kernel::jump_to_user_land(entry);
	}
}
