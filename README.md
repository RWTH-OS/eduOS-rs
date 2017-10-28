# eduOS-rs - A teaching operating system written in Rust

[![Build Status](https://travis-ci.org/RWTH-OS/eduOS-rs.svg?branch=master)](https://travis-ci.org/RWTH-OS/eduOS-rs)

## Introduction

eduOS-rs is a Unix-like computer operating system based on a monolithic architecture for educational purposes, which is developed for the course [Operating Systems][acsos] at the RWTH Aachen Univeristy.
It is derived from following tutorials and software distributions.

1. Philipp Oppermann's [excellent series of blog posts][opp].
2. Erik Kidd's [toyos-rs][kidd], which is an extension of Philipp Opermann's kernel.
3. The original version of [eduOS][stlankes], which was the old teaching kernel written in C.

[opp]: http://blog.phil-opp.com/
[kidd]: http://www.randomhacks.net/bare-metal-rust/
[stlankes]: http://rwth-os.github.io/eduOS/
[rust-barebones-kernel]: https://github.com/thepowersgang/rust-barebones-kernel
[acsos]: http://www.os.rwth-aachen.de/

## Building

Compiling eduOS-rs is tested under Linux and macOS.
For macOS, it is required that Apple's *Command Line Tools* and the package manager [Homebrew](https://brew.sh) are installed.
After installing *Homebrew*, install the required tools *wget*, *nasm* and *qemu* with following command.

```sh
$ brew install wget qemu nasm
```

In addition, you have to install [binutils](https://www.gnu.org/software/binutils/) to support the *Executable and Linkable Format* (ELF), which is the link format of our kernel.
Install these tools as follows:

```sh
$ wget http://ftp.gnu.org/gnu/binutils/binutils-2.29.tar.gz
$ tar xzvf binutils-2.29.tar.gz
$ mkdir build
$ cd build/
$  ../binutils-2.29/configure --prefix=/opt/local/ --target=x86_64-elf --disable-multilib --disable-nls --disable-werror
$ make
$ sudo make install
```

At this point, the build process is identical between Linux and macOS.
It is required to install the Rust toolchain, to check out the sources and to rebuild the Rust runtime using a
bare-metal target without hardware floating point support.

```sh
$ # Set up a Rust compiler. Please use the nightly release channel.
$ curl https://sh.rustup.rs -sSf | sh

$ # Get our source code.
$ git clone git@github.com:RWTH-OS/eduOS-rs.git
$ cd eduOS-rs

$ # Get a copy of the Rust source code so we can rebuild core
$ # for a bare-metal target.
$ git submodule update --init
$ make runtime
```

From here, we should be able to build a kernel and run it within QEMU:

```sh
$ make
$ make run
```

## Overview of all branches

Step by step (here branch by branch) the operating system design will be introduced.
This tutorial shows the steps to develop from a minimal kernel to Unix-like computer operating system.
Currently, following stages of development are available:

0. stage0 - Smallest HelloWorld of the World

   Description of loading a minimal 32bit kernel

1. stage1 - Cooperative/non-preemptive multitasking

   Introduction into a simple form of multitasking, where no interrupts are required.

2. stage2 - Priority-based Cooperative/non-preemptive multitasking

   Introduction into a simple form of priority-based multitasking, where no interrupts are required.

3. stage3 - Synchronization primitives

   Introduce basic synchronization primitives

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

## Licensing

eduOS-rs is licensed under the [MIT license][LICENSE-MIT].

[LICENSE-MIT]: http://opensource.org/licenses/MIT
