/// WIP.  Some bits were sanity-checked against
/// https://github.com/ryanra/RustOS/blob/master/src/arch/x86/idt.rs
///
/// See section 6.10 of
/// http://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-manual-325462.pdf
///
/// See http://jvns.ca/blog/2013/12/04/day-37-how-a-keyboard-works/ for
/// some general advice on setting up interrupts and an entertaining saga
/// of frustration.

use core::mem::size_of;
use core::ptr;
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86;
use x86::irq::IdtEntry;

use arch::x86_64::keyboard;


//=========================================================================
//  Interface to interrupt_handlers.asm

/// Maximum possible number of interrupts; we can shrink this later if we
/// want.
const IDT_ENTRY_COUNT: usize = 256;

#[allow(dead_code)]
extern {
    /// The offset of the main code segment in our GDT.  Exported by our
    /// assembly code.
    static gdt64_code_offset: u16;

    /// A primitive interrupt-reporting function.
    fn report_interrupt();

    /// Interrupt handlers which call back to rust_interrupt_handler.
    static interrupt_handlers: [*const u8; IDT_ENTRY_COUNT];
}

/// Various data available on our stack when handling an interrupt.
///
/// Only `pub` because `rust_interrupt_handler` is.
#[repr(C, packed)]
pub struct InterruptContext {
    rsi: u64,
    rdi: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    int_id: u32,
    _pad_1: u32,
    error_code: u32,
    _pad_2: u32,
}


//=========================================================================
//  Handling interrupts

/// Interface to our PIC (programmable interrupt controller) chips.  We
/// want to map hardware interrupts to 0x20 (for PIC1) or 0x28 (for PIC2).
static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

/// Print our information about a CPU exception, and loop.
fn cpu_exception_handler(ctx: &InterruptContext) {

    // Print general information provided by x86::irq.
    println!("{}, error 0x{:x}",
             x86::irq::EXCEPTIONS[ctx.int_id as usize],
             ctx.error_code);

    // Provide detailed information about our error code if we know how to
    // parse it.
    match ctx.int_id {
        14 => {
            let err = x86::irq::PageFaultError::from_bits(ctx.error_code);
            println!("{:?}", err);
        }
        _ => {}
    }

    loop {}
}

/// Called from our assembly-language interrupt handlers to dispatch an
/// interrupt.
#[no_mangle]
pub unsafe extern "C" fn rust_interrupt_handler(ctx: &InterruptContext) {
    match ctx.int_id {
        0x00...0x0F => cpu_exception_handler(ctx),
        0x20 => { /* Timer. */ }
        0x21 => {
            if let Some(input) = keyboard::read_char() {
                if input == '\r' {
                    println!("");
                } else {
                    print!("{}", input);
                }
            }
        }
        0x80 => println!("Not actually Linux, sorry."),
        _ => {
            println!("UNKNOWN INTERRUPT #{}", ctx.int_id);
            loop {}
        }
    }

    PICS.lock().notify_end_of_interrupt(ctx.int_id as u8);
}


//=========================================================================
//  Interrupt Descriptor Table

/// An Interrupt Descriptor Table which specifies how to respond to each
/// interrupt.
struct Idt {
    table: [IdtEntry; IDT_ENTRY_COUNT],
}

impl Idt {
    /// Initialize interrupt handling.
    pub unsafe fn initialize(&mut self) {
        self.add_handlers();
        self.load();
    }

    /// Fill in our IDT with our handlers.
    fn add_handlers(&mut self) {
        for (index, &handler) in interrupt_handlers.iter().enumerate() {
            if handler != ptr::null() {
                self.table[index] = IdtEntry::new(gdt64_code_offset, handler);
            }
        }
    }

    /// Load this table as our interrupt table.
    unsafe fn load(&self) {
        let pointer = x86::dtables::DescriptorTablePointer {
            base: &self.table[0] as *const IdtEntry as u64,
            limit: (size_of::<IdtEntry>() * IDT_ENTRY_COUNT) as u16,
        };
        x86::dtables::lidt(&pointer);
    }
}

/// Our global IDT.
static IDT: Mutex<Idt> = Mutex::new(Idt {
    table: [missing_handler(); IDT_ENTRY_COUNT]
});


//=========================================================================
//  Initialization

/// Use the `int` instruction to manually trigger an interrupt without
/// actually using `sti` to enable interrupts.  This is highly recommended by
/// http://jvns.ca/blog/2013/12/04/day-37-how-a-keyboard-works/
#[allow(dead_code)]
pub unsafe fn test_interrupt() {
    println!("Triggering interrupt.");
    int!(0x80);
    println!("Interrupt returned!");
}

/// Platform-independent initialization.
pub unsafe fn initialize() {
    PICS.lock().initialize();
    IDT.lock().initialize();

    // Enable this to trigger a sample interrupt.
    test_interrupt();

    // Turn on real interrupts.
    x86::irq::enable();
}


//-------------------------------------------------------------------------
//  Being merged upstream
//
//  This code will go away when https://github.com/gz/rust-x86/pull/4
//  is merged.

/// Create a IdtEntry marked as "absent".  Not tested with real
/// interrupts yet.  This contains only simple values, so we can call
/// it at compile time to initialize data structures.
const fn missing_handler() -> IdtEntry {
    IdtEntry {
        base_lo: 0,
        sel: 0,
        res0: 0,
        flags: 0,
        base_hi: 0,
        res1: 0,
    }
}

trait IdtEntryExt {
    fn new(gdt_code_selector: u16, handler: *const u8) -> IdtEntry;
}

impl IdtEntryExt for IdtEntry {

    /// Create a new IdtEntry pointing at `handler`.
    fn new(gdt_code_selector: u16, handler: *const u8) -> IdtEntry {
        IdtEntry {
            base_lo: ((handler as u64) & 0xFFFF) as u16,
            sel: gdt_code_selector,
            res0: 0,
            flags: 0b100_01110,
            base_hi: (handler as u64) >> 16,
            res1: 0,
        }
    }
}
