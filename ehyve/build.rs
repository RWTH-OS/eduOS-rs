#[cfg(target_os = "linux")]
extern crate cc;

fn main() {
	#[cfg(target_os = "linux")]
	cc::Build::new()
        .file("src/linux/kvm.c")
        .compile("kvm");
}
