---
layout: default
title : eduOS-rs
---

<style>#forkongithub a{background:#000;color:#fff;text-decoration:none;font-family:arial,sans-serif;text-align:center;font-weight:bold;padding:5px 40px;font-size:1rem;line-height:2rem;position:relative;transition:0.5s;}#forkongithub a:hover{background:#c11;color:#fff;}#forkongithub a::before,#forkongithub a::after{content:"";width:100%;display:block;position:absolute;top:1px;left:0;height:1px;background:#fff;}#forkongithub a::after{bottom:1px;top:auto;}@media screen and (min-width:800px){#forkongithub{position:absolute;display:block;top:0;right:0;width:200px;overflow:hidden;height:200px;z-index:9999;}#forkongithub a{width:200px;position:absolute;top:60px;right:-60px;transform:rotate(45deg);-webkit-transform:rotate(45deg);-ms-transform:rotate(45deg);-moz-transform:rotate(45deg);-o-transform:rotate(45deg);box-shadow:4px 4px 10px rgba(0,0,0,0.8);}}</style><span id="forkongithub"><a href="https://github.com/RWTH-OS/eduOS-rs">Fork me on GitHub</a></span>

## A teaching operating system written in Rust

### Introduction

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

### Building

First, we need to check out the source and rebuild the Rust runtime using a
bare-metal target and no hardware floating point support:

```sh
# Set up a Rust compiler. Please use the nightly release channel.
curl https://sh.rustup.rs -sSf | sh

# Get our source code.
git clone git@github.com:RWTH-OS/eduOS-rs.git
cd eduOS-rs

# Get a copy of the Rust source code so we can rebuild core
# for a bare-metal target.
git submodule update --init
make runtime
```

From here, we should be able to build a kernel and run it using QEMU:

```sh
make run
```

### Overview of all branches

Step by step (here branch by branch) the operating system design will be introduced.
This tutorial shows the steps to develop from a minimal kernel to Unix-like computer operating system.
Currently, following stages of development are available:

0. stage0 - Smallest HelloWorld of the World 

   Description of loading a minimal 32bit kernel

### Usefull Links

1. [http://www.gnu.org/software/grub/manual/multiboot/](http://www.gnu.org/software/grub/manual/multiboot/)
2. [http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial](http://www.osdever.net/tutorials/view/brans-kernel-development-tutorial)
3. [http://www.jamesmolloy.co.uk/tutorial_html/index.html](http://www.jamesmolloy.co.uk/tutorial_html/index.html)
4. [http://techblog.lankes.org/tutorials/](http://techblog.lankes.org/tutorials/)
5. [http://www.os.rwth-aachen.de](http://www.os.rwth-aachen.de)
6. [http://www.noteblok.net/2014/06/14/bachelor](http://www.noteblok.net/2014/06/14/bachelor)
7. [https://sourceware.org/newlib/](https://sourceware.org/newlib/)
8. [http://rwth-os.github.io/eduOS/][http://rwth-os.github.io/eduOS/]

### Licensing

eduOS-rs is licensed under the [MIT license][LICENSE-MIT].

[LICENSE-MIT]: http://opensource.org/licenses/MIT

### Contact

* Stefan Lankes, [Institute for Automation of Complex Power Systems](http://www.acs.eonerc.rwth-aachen.de/), [E.ON Energy Research Center](http://www.eonerc.rwth-aachen.de/), [RWTH Aachen University](http://www.rwth-aachen.de/), Germany, E-mail: <slankes@eonerc.rwth-aachen.de>
