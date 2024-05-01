#!/bin/bash
cargo run
if [ $? -eq 3 ]; then
  echo "eduOS-rs runs succesfully within Qemu"
  exit 0
else
  echo "eduOS-rs isn't able to run within Qemu"
  exit 1
fi
