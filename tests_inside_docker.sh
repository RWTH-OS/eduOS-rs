#!/bin/bash
OS_NAME=$1
OS_VERSION=$2

cd /rwth-os/eduOS-rs/

if [ "$OS_NAME" = "centos" ]; then

export PATH=/usr/libexec:$PATH

# Clean the yum cache
yum -y clean all
yum -y clean expire-cache

# First, install all the needed packages.
yum install -y curl wget nasm make qemu-kvm gcc gcc-c++

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
export PATH=$PATH:~/.cargo/bin
cargo install cargo-xbuild bootimage
rustup component add rust-src
cargo install --git https://github.com/RWTH-OS/ehyve.git
make

elif [ "$OS_NAME" = "ubuntu" ]; then

apt-get -qq update
apt-get install -y curl wget qemu-system-x86 nasm make gcc g++ build-essential

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
export PATH=$PATH:~/.cargo/bin
cargo install cargo-xbuild bootimage
rustup component add rust-src
cargo install --git https://github.com/RWTH-OS/ehyve.git
make

fi
