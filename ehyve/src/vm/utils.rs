use std::fs::{File, remove_file};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::fmt;

use nix::unistd::{mkstemp, close};
use raw_cpuid::CpuId;

use vm::error::*;
use vm::kvm::*;

pub unsafe fn any_as_u8_mut_slice<T: Sized>(p: &mut T) -> &mut [u8] {
    ::std::slice::from_raw_parts_mut(
        (p as *mut T) as *mut u8,
        ::std::mem::size_of::<T>()
    )
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>()
    )
}

/// Returns the CPU frequency
pub fn cpufreq() -> Result<u32> {
    let cpuid = CpuId::new();

    if let Some(freq) = cpuid.get_processor_frequency_info() {
        return Ok(freq.processor_base_frequency() as u32);
    }

    let mut content = String::new();

    // If the file cpuinfo_max_freq exists, parse the content and return the frequency
    if let Ok(mut file) = File::open("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        file.read_to_string(&mut content).map_err(|_| Error::MissingFrequency)?;
        return content.trim().parse::<u32>().map_err(|_| Error::MissingFrequency).map(|x| x / 1000);
    }
    // otherwise use the more acurate cpuinfo file and search for the right line
    else if let Ok(mut file) = File::open("/proc/cpuinfo") {
        file.read_to_string(&mut content).expect("Couldnt read!");

        for line in content.lines() {
            if line.starts_with("cpu MHz") {
                return line.split(':').skip(1).next().ok_or(Error::MissingFrequency)?
                    .trim().parse::<f32>().map_err(|_| Error::MissingFrequency).map(|x| x as u32);
            }
        }
    }

    // ups shouldn't happened ..
    Err(Error::MissingFrequency)
}

pub fn parse_mem(mem: &str) -> Result<u64> {
    let (num, postfix): (String, String) = mem.chars().partition(|&x| x.is_numeric());
    let num = num.parse::<u64>().map_err(|_| Error::ParseMemory)?;

    let factor = match postfix.as_str() {
        "E" | "e" => 1 << 60,
        "P" | "p" => 1 << 50,
        "T" | "t" => 1 << 40,
        "G" | "g" => 1 << 30,
        "M" | "m" => 1 << 20,
        "K" | "k" => 1 << 10,
        _ => return Err(Error::ParseMemory)
    };

    Ok(num*factor)
}

#[derive(Debug)]
pub struct TmpFile {
    path: PathBuf
}

impl TmpFile {
    pub fn create(name: &str) -> Result<TmpFile> {
        match mkstemp(name) {
            Ok((fd, path)) => {
                close(fd).map_err(|_| Error::CannotCreateTmpFile)?;
                debug!("Created tmp file with name {}", path.display());
                Ok(TmpFile { path: path })
            },
            Err(_) => Err(Error::CannotCreateTmpFile)
        }
    }

    pub fn read_to_string(&self, buf: &mut String) -> Result<usize> {
        match File::open(self.path.as_path()) {
            Ok(mut file) => {
                Ok(file.read_to_string(buf).map_err(|_| Error::CannotReadTmpFile(format!("{}", self.path.display())))?)
            },
            Err(_) => Err(Error::CannotReadTmpFile(format!("{}", self.path.display())))
        }
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn delete(&self) {
        match remove_file(self.path.as_path()) {
            Ok(_) => debug!("Deleted tmp file {}", self.path.display()),
            Err(_) => {}
        };
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        self.delete();
    }
}

impl fmt::Display for kvm_regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "rip: {:016x}   rsp: {:016x} flags: {:016x}\n\
            rax: {:016x}   rbx: {:016x}   rcx: {:016x}\n\
            rdx: {:016x}   rsi: {:016x}   rdi: {:016x}\n\
            rbp: {:016x}    r8: {:016x}    r9: {:016x}\n\
            r10: {:016x}   r11: {:016x}   r12: {:016x}\n\
            r13: {:016x}   r14: {:016x}   r15: {:016x}\n",
            self.rip, self.rsp, self.rflags,
            self.rax, self.rbx, self.rcx,
            self.rdx, self.rsi, self.rdi,
            self.rbp, self.r8,  self.r9,
            self.r10, self.r11, self.r12,
            self.r13, self.r14, self.r15
        )
    }
}

impl fmt::Display for kvm_sregs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"cr0: {:016x}   cr2: {:016x}   cr3: {:016x}\ncr4: {:016x}   cr8: {:016x}\n",
            self.cr0, self.cr2, self.cr3, self.cr4, self.cr8)
    }
}

impl fmt::Display for kvm_dtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:016x}  {:08x}", self.base, self.limit)
    }
}

impl fmt::Display for kvm_segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
            self.selector, self.base, self.limit, self.type_, self.present, self.dpl, self.db, self.s, self.l, self.g, self.avl)
    }
}

fn show_dtable(name: &str, dtable: &kvm_dtable) {
    print!("{}                 {}\n", name, dtable);
}

fn show_segment(name: &str, seg: &kvm_segment) {
    print!("{}       {}\n", name, seg);
}

pub fn show_registers(id: u32, regs: &kvm_regs, sregs: &kvm_sregs) {
    print!("\nDump state of CPU {}\n", id);
    print!("\nRegisters:\n");
	print!("----------\n");
    print!("{}{}", regs, sregs);

    print!("\nSegment registers:\n");
	print!("------------------\n");
	print!("register  selector  base              limit     type  p dpl db s l g avl\n");
    show_segment("cs ", &sregs.cs);
    show_segment("ss ", &sregs.ss);
    show_segment("ds ", &sregs.ds);
    show_segment("es ", &sregs.es);
    show_segment("fs ", &sregs.fs);
    show_segment("gs ", &sregs.gs);
    show_segment("tr ", &sregs.tr);
    show_segment("ldt", &sregs.ldt);
    show_dtable("gdt", &sregs.gdt);
    show_dtable("gdt", &sregs.idt);

    print!("\nAPIC:\n");
	print!("-----\n");
	print!("efer: {:016x}  apic base: {:016x}\n", sregs.efer, sregs.apic_base);

    print!("\nInterrupt bitmap:\n");
	print!("-----------------\n");

    for i in 0 .. (KVM_NR_INTERRUPTS + 63) / 64 {
        print!("{:016x} ", sregs.interrupt_bitmap[i as usize]);
    }
    print!("\n")
}
