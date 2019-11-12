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

ifeq ($(arch), x86_64)
BUILD_COMMAD := cargo bootimage $(opt) --target $(target).json
RUN_COMMAND := bootimage run $(opt) --target $(target).json || ([ $$? -eq 1 ] && exit 0) || exit 1
else
BUILD_COMMAD := cargo xbuild $(opt) --target $(target).json
RUN_COMMAND := qemu-system-aarch64 -semihosting -M virt -cpu cortex-a53 -m 1G -serial stdio -display none -kernel target/$(arch)-eduos/$(rdir)/eduos-rs || ([ $$? -eq 1 ] && exit 0) || exit 1
endif

.PHONY: all build fmt clean run debug cargo docs

all: build $(build_wasm)

build:
	@$(BUILD_COMMAD)

fmt:
	rustfmt --write-mode overwrite src/lib.rs

qemu:
	@$(RUN_COMMAND)

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
