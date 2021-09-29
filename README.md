[![Build Status](https://travis-ci.com/RWTH-OS/eduOS-rs.svg?branch=master)](https://travis-ci.com/RWTH-OS/eduOS-rs)
![Actions Status](https://github.com/RWTH-OS/eduOS-rs/workflows/Build/badge.svg)

# eduOS-rs - A teaching operating system written in Rust

## Introduction

<p align="center"><img src="/img/demo.gif?raw=true"/></p>

eduOS-rs is a Unix-like operating system based on a monolithic architecture for educational purposes.
It is developed for the course [Operating Systems][acsos] at RWTH Aachen University and includes a modified hypervisor that simplifies the boot process to increase the intelligibility of the OS.
eduOS-rs is derived from following tutorials and software distributions:

1. Philipp Oppermann's [excellent series of blog posts][opp].
2. Erik Kidd's [toyos-rs][kidd], which is an extension of Philipp Opermann's kernel.
3. The original version of [eduOS][stlankes], which was the old teaching kernel written in C.

[opp]: http://blog.phil-opp.com/
[kidd]: http://www.randomhacks.net/bare-metal-rust/
[stlankes]: http://rwth-os.github.io/eduOS/
[rust-barebones-kernel]: https://github.com/thepowersgang/rust-barebones-kernel
[acsos]: http://www.os.rwth-aachen.de/

## Requirements to build eduOS-rs

eduOS-rs is tested under Linux, macOS, and Windows.

### macOS

Apple's *Command Line Tools* must be installed.
The Command Line Tool package gives macOS terminal users many commonly used tools and compilers, that are usually found in default Linux installations.
Following terminal command installs these tools without Apple's IDE Xcode:

```sh
$ xcode-select --install
```

### Windows

To build eduOS-rs you have to install a linker and a [git client](https://git-scm.com/downloads).
We tested the eduOS-rs with the linker from Visual Studio.
Consequently, we suggest installing Visual Studio in addition to [git](https://git-scm.com/downloads).

### Linux

Linux users should install common developer tools.
For instance, on Ubuntu 18.04 the following command installs the required tools:

```sh
$ apt-get install -y git curl wget nasm make autotools-dev gcc g++ build-essential lld-8
```

### Common for macOS, Windows and Linux
This project uses Rustup to set its Rust toolchain.
Follow the instructions to [install Rust using Rustup](https://www.rust-lang.org/tools/install).

eduOS-rs is able to run within [ehyve](https://github.com/RWTH-OS/ehyve), which a specialized hypervisor for eduOS-rs.
Therefore [ehyve](https://github.com/RWTH-OS/ehyve) must be installed.

```sh
$ cargo +nightly install --git https://github.com/RWTH-OS/ehyve.git --locked
```

Please check if your system fullfil ehyve's [system requirements](https://github.com/RWTH-OS/ehyve).

## Building

After cloning the repository, you can run the kernel with ehyve:

```sh
$ cargo run
```

## Overview of all branches

Step by step (here branch by branch) the operating system design will be introduced.
This tutorial shows the steps to develop from a minimal kernel to a Unix-like computer operating system.
Currently, following stages of development are available:

0. stage0 - Smallest HelloWorld of the World

   Description of loading a minimal 64bit kernel

1. stage1 - Cooperative/non-preemptive multitasking

   Introduction into a simple form of multitasking, where no interrupts are required.

2. stage2 - Priority-based cooperative/non-preemptive multitasking

   Introduction into a simple form of priority-based multitasking, where no interrupts are required.

3. stage3 - Synchronization primitives

   Introduce basic synchronization primitives

4. stage 4 - Preemptive multitasking

   Introduction into preemptive multitasking and interrupt handling

5. stage 5 - Support of user-level tasks

   Add support of user-level tasks with an small interface for basic system calls

6. stage 6 - Support of paging

   Add support of paging and a simple demo for process creation

7. stage 7 - Integration of an in-memory file system

   Introduce a virtual file system with an in-memory file system as example file system.

8. stage8 - Run Linux application as common process

   Start a simple Linux application (_HelloWorld_) on top of eduOS-rs. The application is a _position-independent executable_ (PIE) and use [musl-libc](http://www.musl-libc.org) as standard C library.

## Useful Links

1. [http://www.gnu.org/software/grub/manual/multiboot/](http://www.gnu.org/software/grub/manual/multiboot/)
2. [http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial](http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial)
3. [http://www.jamesmolloy.co.uk/tutorial_html/index.html](http://www.jamesmolloy.co.uk/tutorial_html/index.html)
4. [http://techblog.lankes.org/tutorials/](http://techblog.lankes.org/tutorials/)
5. [http://www.os.rwth-aachen.de](http://www.os.rwth-aachen.de)
6. [http://www.noteblok.net/2014/06/14/bachelor](http://www.noteblok.net/2014/06/14/bachelor)
7. [https://sourceware.org/newlib/](https://sourceware.org/newlib/)
8. [http://rwth-os.github.io/eduOS/](http://rwth-os.github.io/eduOS/)
9. [https://intermezzos.github.io](https://intermezzos.github.io)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
