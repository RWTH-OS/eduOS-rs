pub mod error;
pub mod ehyve;
pub mod vcpu;

use libc::{c_int, c_void};

extern {
	fn setup_guest_mem(mem: *mut c_void) -> c_int;
}
