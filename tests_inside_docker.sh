#!/bin/bash

OS_NAME=$1
OS_VERSION=$2

cd /rwth-os/eduOS-rs/

if [ "$OS_NAME" = "centos" ]; then

# Clean the yum cache
echo "multilib_policy=all" >> /etc/yum.conf
yum -y clean all
yum -y clean expire-cache

# First, install all the needed packages.
yum install -y curl wget qemu-system-x86 nasm make autotools gcc gcc-c++ glibc-devel.i686

elif [ "$OS_NAME" = "ubuntu" ]; then

apt-get -qq update
apt-get install -y curl wget qemu-system-x86 nasm make autotools-dev gcc g++ build-essential g++-multilib

fi

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
export PATH=$PATH:~/.cargo/bin

# tests in 64bit mode
make runtime
make
make run

# tests in 32bit mode
make arch=i686 runtime
make arch=i686
make arch=i686 run
