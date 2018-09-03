#[cfg(target_os = "linux")]
extern crate gcc;

fn main() {
	#[cfg(target_os = "linux")]
    gcc::compile_library("libkvm.a", &["src/linux/kvm.c"]);
}
