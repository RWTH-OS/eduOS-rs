# Copied from http://blog.phil-opp.com/rust-os/multiboot-kernel.html

arch ?= x86_64
target ?= $(arch)-unknown-none-gnu
archdir ?= x86
release ?= 1

opt :=
rdir := debug
nasmflags = -felf64

ifeq ($(release), 1)
opt := --release
rdir := release
endif

rust_os := target/$(target)/$(rdir)/libeduos_rs.a
kernel := build/kernel-$(arch)

crossprefix :=
uname_s := $(shell uname -s)

ifeq ($(uname_s),Darwin)
crossprefix += x86_64-elf-
endif

linker_script := src/arch/$(archdir)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_header_files := $(wildcard src/arch/$(archdir)/*.inc)
assembly_source_files := $(wildcard src/arch/$(archdir)/*.asm)
assembly_object_files := $(patsubst src/arch/$(archdir)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

ld_for_target := $(crossprefix)ld
objcopy_for_target := $(crossprefix)objcopy
strip_debug := --strip-debug
keep_debug := --only-keep-debug
output_format := -O elf32-i386

.PHONY: all fmt clean run debug cargo docs

all: $(kernel).elf

fmt:
	rustfmt --write-mode overwrite src/lib.rs

clean:
	rm -rf build target

run: $(kernel).elf
	@echo QEMU $(kernel).elf
	@qemu-system-x86_64 -display none -smp 1 -net nic,model=rtl8139 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -monitor telnet:127.0.0.1:18767,server,nowait -kernel $(kernel).elf -serial stdio 2>/dev/null || true

debug: $(kernel).elf
	@echo QEMU -d int $(kernel).elf
	@qemu-system-x86_64 -display none -smp 1 -net nic,model=rtl8139 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -monitor telnet:127.0.0.1:18767,server,nowait -kernel $(kernel).elf -d int -no-reboot -serial stdio

$(kernel).elf: cargo $(assembly_object_files) $(linker_script)
	@echo LD $(kernel).elf
	@$(ld_for_target) -n --gc-sections -T $(linker_script) -o $(kernel).elf \
		$(assembly_object_files) $(rust_os)
	@echo OBJCOPY $(kernel).sym
	@$(objcopy_for_target) $(keep_debug) $(kernel).elf $(kernel).sym
	@echo OBJCOPY $(kernel).elf
	@$(objcopy_for_target) $(strip_debug) $(output_format) $(kernel).elf

docs:
	@echo DOC
	@cargo doc

cargo:
	@echo CARGO
	@cargo build $(opt) --target $(target)

build/arch/$(arch)/%.o: src/arch/$(archdir)/%.asm $(assembly_header_files)
	@echo NASM $<
	@mkdir -p $(shell dirname $@)
	@nasm $(nasmflags) -Isrc/arch/$(archdir)/ $< -o $@


#==========================================================================
# Building the Rust runtime for our bare-metal target

# Where to put our compiled runtime libraries for this platform.
installed_target_libs := \
	$(shell rustup which rustc | \
		sed s,bin/rustc,lib/rustlib/$(target)/lib,)

runtime_rlibs := \
	$(installed_target_libs)/libcore.rlib \
	$(installed_target_libs)/libstd_unicode.rlib \
	$(installed_target_libs)/liballoc.rlib

RUSTC := \
	rustc --verbose --target $(target) \
		-Z no-landing-pads \
		--out-dir $(installed_target_libs)

.PHONY: runtime

runtime: $(runtime_rlibs)

$(installed_target_libs):
	@mkdir -p $(installed_target_libs)

$(installed_target_libs)/%.rlib: rust/src/%/lib.rs $(installed_target_libs)
	@echo RUSTC $<
	@$(RUSTC) --crate-type rlib --crate-name $(shell basename $@ | sed s,lib,, | sed s,.rlib,,) $<
	@echo Check $@
