# eduOS-rs - A teaching operating system written in Rust

[![Build Status](https://travis-ci.org/RWTH-OS/eduOS-rs.svg?branch=master)](https://travis-ci.org/RWTH-OS/eduOS-rs)

## Introduction

eduOS-rs is a Unix-like operating system based on a monolithic architecture for educational purposes.
It is developed for the course [Operating Systems][acsos] at the RWTH Aachen University and includes a modified hypervisor that simplifies the boot process to increase intelligibility of the OS.
eduOS-rs is derived from following tutorials and software distributions.

1. Philipp Oppermann's [excellent series of blog posts][opp].
2. Erik Kidd's [toyos-rs][kidd], which is an extension of Philipp Opermann's kernel.
3. The original version of [eduOS][stlankes], which was the old teaching kernel written in C.

[opp]: http://blog.phil-opp.com/
[kidd]: http://www.randomhacks.net/bare-metal-rust/
[stlankes]: http://rwth-os.github.io/eduOS/
[rust-barebones-kernel]: https://github.com/thepowersgang/rust-barebones-kernel
[acsos]: http://www.os.rwth-aachen.de/

## Requirements to build eduOS-rs
eduOS-rs is tested under Linux, macOS and Windows.

### macOS
Apple's *Command Line Tools* must be installed.
The Command Line Tool package gives macOS terminal users many commonly used tools, and compilers, that are usually found in default Linux installations.
Following terminal command installs these tools without Apple's IDE Xcode:

```sh
$ xcode-select --install
```

### Windows
To build eduOS-rs you must install a linker, [make](http://gnuwin32.sourceforge.net/packages/make.htm) and a [git client](https://git-scm.com/downloads). We tested the eduOS-rs with the linker from Visual Studio.
Consequently, we suggest you install Visual Studio in addition to [make](http://gnuwin32.sourceforge.net/packages/make.htm) and [git](https://git-scm.com/downloads).

### Linux
Linux users should install typical developer tools.
For instance, on Ubuntu 18.04 following command is used to install the required tools.

```sh
$ apt-get install -y curl wget nasm make autotools-dev gcc g++ build-essential
```

### Common for macOS, Windows and Linux
It is required to install the Rust toolchain.
Please visit the [Rust website](https://www.rust-lang.org/) and follow the installation instructions for your operating system. It is important that the *nightly channel* is used to install the toolchain.
This is queried during installation and should be answered accordingly.

Afterwards the installation of *cargo-xbuild* and source code of Rust runtime are required to build the kernel.

```sh
$ cargo install cargo-xbuild
$ rustup component add rust-src
```

## Building
The final step is to create a copy of the repository and to build the kernel.

```sh
$ # Get our source code.
$ git clone -b stage0 git@github.com:RWTH-OS/eduOS-rs.git
$ cd eduOS-rs

$ # Get a copy of the Rust source code so we can rebuild core
$ # for a bare-metal target.
$ git submodule update --init
$ make
```

From here, we should be able to run the kernel in eHyve, which is the hypervisor for eduOS-rs and part of this repository.

```sh
$ make run
```

## Overview of all branches

Step by step (here branch by branch) the operating system design will be introduced.
This tutorial shows the steps to develop from a minimal kernel to Unix-like computer operating system.
Currently, following stages of development are available:

0. stage0 - Smallest HelloWorld of the World

   Description of loading a minimal 64bit kernel

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
