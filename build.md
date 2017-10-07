---
layout: page
title : Building eduOS-rs
sidebar_link: true
---

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

eduos-rs' build process is test under Linux and macOS.
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
