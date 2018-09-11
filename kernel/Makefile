arch ?= x86_64
target ?= $(arch)-unknown-unknown
release ?=

opt :=
rdir := debug
suffix := elf

ifeq ($(release), 1)
opt := --release
rdir := release
endif

ifeq ($(arch), wasm32)
suffix := wasm
endif

rust_os := target/$(target)/$(rdir)/libeduos_rs.a
kernel := build/kernel-$(arch)

crossprefix :=
uname_s := $(shell uname -s)

ifeq ($(uname_s),Darwin)
crossprefix += x86_64-elf-
endif

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg

ld_for_target := $(crossprefix)ld
objcopy_for_target := $(crossprefix)objcopy
strip_debug := --strip-debug
keep_debug := --only-keep-debug

.PHONY: all fmt clean run debug cargo docs

all: $(kernel).$(suffix)

fmt:
	rustfmt --write-mode overwrite src/lib.rs

clean:
	rm -rf build target

$(kernel).wasm: xargo
	@echo WASM_GC
	@wasm-gc target/$(target)/$(rdir)/eduos_rs.wasm eduos.wasm

$(kernel).elf: xargo build $(linker_script)
	@echo LD $(kernel).elf
	@$(ld_for_target) -n --gc-sections -T $(linker_script) -o $(kernel).elf $(rust_os)

build:
	mkdir -p build

docs:
	@echo DOC
	@cargo doc

xargo:
	@echo XARGO
	@RUST_TARGET_PATH=${CURDIR} xargo build $(opt) --target $(target)
