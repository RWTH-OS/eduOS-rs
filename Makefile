arch ?= x86_64
target ?= $(arch)-eduos
release ?=

opt :=
rdir := debug

ifeq ($(release), 1)
opt := --release
rdir := release
endif

build_wasm :=
ifeq ($(arch), wasm32)
build_wasm := eduos.wasm
endif

RN :=
ifdef COMSPEC
RM := del
else
RM := rm -rf
endif

.PHONY: all fmt clean run debug cargo docs

all: cargo $(build_wasm)

bootimage.bin:
	bootimage build --target $(target).json

fmt:
	rustfmt --write-mode overwrite src/lib.rs

qemu: bootimage.bin
	@qemu-system-x86_64 -display none -smp 1 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -drive format=raw,file=bootimage.bin || true

run:
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

clean:
	$(RM) target

eduos.wasm: cargo
	@echo WASM_GC
	@wasm-gc target/$(target)/$(rdir)/eduos_rs.wasm eduos.wasm

docs:
	@echo DOC
	@cargo doc

cargo:
	@echo CARGO
	@cargo xbuild $(opt) --target $(target).json
