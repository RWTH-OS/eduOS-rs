extern crate cc;

fn main() {
	#[cfg(target_os = "linux")]
	cc::Build::new()
        .file("src/linux/kvm.c")
        .compile("kvm");

	#[cfg(target_os = "macos")]
	cc::Build::new()
        .file("src/macos/mem.c")
        .compile("mem");
}
