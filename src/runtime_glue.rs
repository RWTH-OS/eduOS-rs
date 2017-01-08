//! Minor functions that Rust really expects to be defined by our compiler
//! or something, but which we need to provide manually because we're on
//! bare metal.

#[lang = "eh_personality"]
extern "C" fn eh_personality() {
}

#[lang = "panic_fmt"] #[no_mangle]
extern "C" fn panic_fmt(
    args: ::core::fmt::Arguments, file: &str, line: usize)
    -> !
{
    println!("PANIC: {}:{}: {}", file, line, args);
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn _Unwind_Resume()
{
    println!("UNWIND!");
    loop {}
}
