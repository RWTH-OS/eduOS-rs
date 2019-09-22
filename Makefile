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
	@cargo bootimage --target $(target).json

fmt:
	rustfmt --write-mode overwrite src/lib.rs

qemu: bootimage.bin
	@bootimage run --target $(target).json || ([ $$? -eq 1 ] && exit 0) || exit 1

run:
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

clean:
	$(RM) target bootimage.bin

eduos.wasm: cargo
	@echo WASM_GC
	@wasm-gc target/$(target)/$(rdir)/eduos_rs.wasm eduos.wasm

docs:
	@echo DOC
	@cargo doc

cargo:
	@echo CARGO
	@cargo xbuild $(opt) --target $(target).json
