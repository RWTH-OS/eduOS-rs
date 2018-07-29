pub static mut VERBOSE: bool = false;

pub fn is_verbose() -> bool {
    return unsafe { VERBOSE };
}
