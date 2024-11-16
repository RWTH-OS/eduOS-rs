#[cfg(target_arch = "x86")]
#[repr(C, align(64))]
pub(crate) struct Aligned<T>(T);

#[cfg(target_arch = "x86")]
impl<T> Aligned<T> {
	/// Constructor.
	pub const fn new(t: T) -> Self {
		Self(t)
	}
}

#[cfg(target_arch = "x86")]
pub(crate) const BOOT_STACK_SIZE: usize = 0x3000;
#[cfg(target_arch = "x86")]
#[link_section = ".data"]
pub(crate) static mut BOOT_STACK: Aligned<[u8; BOOT_STACK_SIZE]> =
	Aligned::new([0; BOOT_STACK_SIZE]);
