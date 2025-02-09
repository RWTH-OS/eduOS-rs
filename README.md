![Actions Status](https://github.com/RWTH-OS/eduOS-rs/workflows/Build/badge.svg)

# eduOS-rs - A teaching operating system written in Rust

## Introduction

eduOS-rs is a Unix-like operating system based on a monolithic architecture for educational purposes.
It is developed for the course [Operating Systems][acsos] at RWTH Aachen University.
eduOS-rs is derived from following tutorials and software distributions:

1. Philipp Oppermann's [excellent series of blog posts][opp].
2. Erik Kidd's [toyos-rs][kidd], which is an extension of Philipp Opermann's kernel.
3. The original version of [eduOS][stlankes], which was the old teaching kernel written in C.
4. eduOS-rs' uses a memory allocator, which is derived from the [buddy_system_allocator][buddy].
5. The first version of paging is derived from a version, which was developed by [Colin Finck][colin].

[opp]: http://blog.phil-opp.com/
[kidd]: http://www.randomhacks.net/bare-metal-rust/
[stlankes]: http://rwth-os.github.io/eduOS/
[rust-barebones-kernel]: https://github.com/thepowersgang/rust-barebones-kernel
[acsos]: https://www.acs.eonerc.rwth-aachen.de/cms/e-on-erc-acs/studium/lehrveranstaltungen/~xwyom/grundgebiete-der-informatik-4-betriebs/?lidx=1
[colin]: https://github.com/ColinFinck
[buddy]: https://github.com/rcore-os/buddy_system_allocator

## Requirements to build eduOS-rs
eduOS-rs is tested under Linux, macOS, and Windows.

### macOS
Apple's *Command Line Tools* must be installed.
The Command Line Tool package gives macOS terminal users many commonly used tools and compilers, that are usually found in default Linux installations.
Following terminal command installs these tools without Apple's IDE Xcode:

```sh
$ xcode-select --install
```

In addition, *Qemu* must be installed.
Please use [Homebrew](https://brew.sh) as package manager to install Qemu.

```sh
$ brew install qemu 
```

### Windows
To build eduOS-rs you have to install _Qemu_ and a git client.
Please use [Chocolatey](https://chocolatey.org) as package manager to install Qemu and git.

```sh
$ choco install qemu git
```

### Linux
Linux users should install common developer tools.
For instance, on Ubuntu 22.04 the following command installs the required tools:

```sh
$ apt-get install -y git qemu-system-x86 build-essential
```

### Common for macOS, Windows and Linux
This project uses Rustup to set its Rust toolchain.
Follow the instructions to [install Rust using Rustup](https://www.rust-lang.org/tools/install).

In addition, the tool [bootimage](https://github.com/rust-osdev/bootimage) is required, which creates a bootable diskimage.
Please install the tool with following command.

```sh
$ cargo install bootimage
```

## Building

eduOS-rs is able to run within [Qemu](https://www.qemu.org), which is a generic and open source machine emulator and virtualizer.

After cloning the repository, you can run the kernel with following command:

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

7. stage 7 - Basic IO interface

   Add basic support of file descriptors

8. stage 8 - Integration of an in-memory file system

   Introduce a virtual file system with an in-memory file system as example file system.

9. stage9 - Run Linux application as common process

   Start a simple Linux application (_HelloWorld_) on top of eduOS-rs. The application is a _position-independent executable_ (PIE) and use [musl-libc](http://www.musl-libc.org) as standard C library. The FPU support is disabled.

## Useful Links

1. [http://www.gnu.org/software/grub/manual/multiboot/](http://www.gnu.org/software/grub/manual/multiboot/)
2. [http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial](http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial)
3. [https://www.acs.eonerc.rwth-aachen.de/cms/e-on-erc-acs/studium/lehrveranstaltungen/~xwyom/grundgebiete-der-informatik-4-betriebs/?lidx=1](https://www.acs.eonerc.rwth-aachen.de/cms/e-on-erc-acs/studium/lehrveranstaltungen/~xwyom/grundgebiete-der-informatik-4-betriebs/?lidx=1)
4. [http://rwth-os.github.io/eduOS/](http://rwth-os.github.io/eduOS/)
5. [https://intermezzos.github.io](https://intermezzos.github.io)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
