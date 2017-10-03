#!/bin/bash

OS_NAME=$1
OS_VERSION=$2

cd /rwth-os/eduOS-rs/

if [ "$OS_NAME" = "centos" ]; then

# Clean the yum cache
yum -y clean all
yum -y clean expire-cache

# First, install all the needed packages.
yum install -y curl wget qemu-system-x86 nasm make

elif [ "$OS_NAME" = "ubuntu" ]; then

apt-get -qq update
apt-get install -y curl wget qemu-system-x86 nasm make

fi

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
export PATH=$PATH:~/.cargo/bin
make runtime
make
make run
