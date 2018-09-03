use std::fmt;
use linux::*;

impl fmt::Display for Regs {
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

impl fmt::Display for Sregs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"cr0: {:016x}   cr2: {:016x}   cr3: {:016x}\ncr4: {:016x}   cr8: {:016x}\n",
            self.cr0, self.cr2, self.cr3, self.cr4, self.cr8)
    }
}

impl fmt::Display for Dtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:016x}  {:08x}", self.base, self.limit)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
            self.selector, self.base, self.limit, self.type_, self.present, self.dpl, self.db, self.s, self.l, self.g, self.avl)
    }
}

fn show_dtable(name: &str, dtable: &Dtable) {
    print!("{}                 {}\n", name, dtable);
}

fn show_segment(name: &str, seg: &Segment) {
    print!("{}       {}\n", name, seg);
}

pub fn show_registers(id: u32, regs: &Regs, sregs: &Sregs) {
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
    show_dtable("idt", &sregs.idt);

    print!("\nAPIC:\n");
	print!("-----\n");
	print!("efer: {:016x}  apic base: {:016x}\n", sregs.efer, sregs.apic_base);

    /*print!("\nInterrupt bitmap:\n");
	print!("-----------------\n");

    for i in 0 .. (KVM_NR_INTERRUPTS + 63) / 64 {
        print!("{:016x} ", sregs.interrupt_bitmap[i as usize]);
    }*/
    print!("\n")
}
