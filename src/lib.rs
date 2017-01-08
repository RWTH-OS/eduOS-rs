#![feature(asm, const_fn, lang_items, unique, collections)]
#![no_std]

extern crate collections;

extern crate alloc_buddy_simple;
extern crate cpuio;
extern crate pic8259_simple;
extern crate rlibc;
extern crate spin;

#[macro_use(int)]
extern crate x86;

use core::fmt::Write;

// These need to be visible to the linker, so we need to export them.
pub use arch::interrupts::rust_interrupt_handler;
pub use runtime_glue::*;

#[macro_use]
mod macros;
mod runtime_glue;
mod heap;
mod arch;
mod console;

#[no_mangle]
pub extern "C" fn rust_main() {
    use arch::vga::{SCREEN, ColorScheme};
    use arch::vga::Color::*;

    SCREEN.lock()
          .clear(DarkGrey)
          .set_colors(ColorScheme::new(Yellow, DarkGrey));
    println!("Hello, world!");

    unsafe {
        arch::interrupts::initialize();
        heap::initialize();
    }

    let mut vec = collections::vec::Vec::<u8>::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    println!("Hey, I made a vector in kernel space! {:?}", vec);

    println!("Scanning PCI bus...");
    for function in arch::pci::functions() {
        println!("{}", function);
    }

    arch::cpuid::cpu_init();
    arch::cpuid::print_infos();

    println!("Running.");

    loop {}
}
