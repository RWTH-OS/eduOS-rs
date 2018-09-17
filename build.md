## Requirements to build eduOS-rs
eduOS-rs is tested under Linux, macOS and Windows.

### macOS
Apple's *Command Line Tools* must be installed.
The Command Line Tool package gives macOS terminal users many commonly used tools, and compilers, that are usually found in default Linux installations.
Following terminal command installs these tools without Apple's IDE Xcode:

```sh
$ xcode-select --install
```

In addition, the included hypervisor based on the [Hypervisor Framework](https://developer.apple.com/documentation/hypervisor), which depends on OS X Yosemite (10.10), or newer.
Please activate this feature as *root* with following command on your system:

```sh
$ sysctl kern.hv_support=1
```

### Windows
To build eduOS-rs you must install a linker, [make](http://gnuwin32.sourceforge.net/packages/make.htm) and a [git client](https://git-scm.com/downloads). We tested the eduOS-rs with the linker from Visual Studio.
Consequently, we suggest you to install Visual Studio in addition to [make](http://gnuwin32.sourceforge.net/packages/make.htm) and [git](https://git-scm.com/downloads).

In addition, the included hypervisor based on the [Windows Hypervisor Platform](https://docs.microsoft.com/en-us/virtualization/api/), which depends on Windows 10 (build 17134 or above) or Windows Server (1803 or above).
Please activate this feature as *root* with following command on your system:

```sh
Dism /Online /Enable-Feature /FeatureName:HypervisorPlatform
```

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
