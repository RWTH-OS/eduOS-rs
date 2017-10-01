# eduOS-rs - A teaching operating system written in Rust

[![Build Status](https://travis-ci.org/RWTH-OS/eduOS-rs.svg?branch=master)](https://travis-ci.org/RWTH-OS/eduOS-rs)

## Introduction

eduOS-rs is a computer operating system based on a monolithic architecture for educational purposes.
It is derived from following tutorials and software distributions.

1. Philipp Oppermann's [excellent series of blog posts][opp].
2. Erik Kidd's [toyos-rs][kidd], which is an extension of Philipp Opermann's kernel.
3. Stefan Lankes [eduos][stlankes], which is the old teaching kernel written in C.

[opp]: http://blog.phil-opp.com/
[kidd]: http://www.randomhacks.net/bare-metal-rust/
[stlankes]: http://rwth-os.github.io/eduOS/
[rust-barebones-kernel]: https://github.com/thepowersgang/rust-barebones-kernel

## Building

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

You should be able to type.

## Licensing

eduOS-rs is licensed under the [MIT license][LICENSE-MIT].

[LICENSE-MIT]: http://opensource.org/licenses/MIT
