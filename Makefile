# Copied from http://blog.phil-opp.com/rust-os/multiboot-kernel.html

arch ?= x86_64
target ?= $(arch)-unknown-none-gnu

rust_os := target/$(target)/debug/libtoyos.a
kernel := build/kernel-$(arch)

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_header_files := $(wildcard src/arch/$(arch)/*.inc)
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

crossprefix :=
objcopy_for_target := $(crossprefix)objcopy
strip_debug := --strip-debug
keep_debug := --only-keep-debug
output_format := -O elf32-i386

.PHONY: all fmt clean run debug cargo

all: $(kernel).elf

fmt:
	rustfmt --write-mode overwrite src/lib.rs

clean:
	rm -rf build target

run: $(kernel).elf
	@echo QEMU $(kernel).elf
	@qemu-system-x86_64 -kernel $(kernel).elf -serial stdio

debug: $(kernel).elf
	@echo QEMU -d int $(kernel).elf
	@qemu-system-x86_64 -kernel $(kernel).elf -d int -no-reboot -serial stdio

$(kernel).elf: cargo $(assembly_object_files) $(linker_script)
	@echo LD $(kernel).elf
	@ld -n --gc-sections -T $(linker_script) -o $(kernel).elf \
		$(assembly_object_files) $(rust_os)
	@echo OBJCOPY $(kernel).sym
	@$(objcopy_for_target) $(keep_debug) $(kernel).elf $(kernel).sym
	@echo OBJCOPY $(kernel).elf
	@$(objcopy_for_target) $(strip_debug) $(output_format) $(kernel).elf

cargo:
	@echo CARGO
	@cargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm $(assembly_header_files)
	@echo NASM $<
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 -Isrc/arch/$(arch)/ $< -o $@


#==========================================================================
# Building the Rust runtime for our bare-metal target

# Where to put our compiled runtime libraries for this platform.
installed_target_libs := \
	$(shell rustup which rustc | \
		sed s,bin/rustc,lib/rustlib/$(target)/lib,)

runtime_rlibs := \
	$(installed_target_libs)/libcore.rlib \
	$(installed_target_libs)/liballoc.rlib \
	$(installed_target_libs)/libstd_unicode.rlib \
	$(installed_target_libs)/librustc_unicode.rlib \
	$(installed_target_libs)/libcollections.rlib

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
	@$(RUSTC) $<
	@echo Check $(installed_target_libs)
