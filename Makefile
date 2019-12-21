arch ?= x86_64
target ?= $(arch)-eduos
release ?= 0

opt :=
rdir := debug

ifeq ($(release), 1)
opt := --release
rdir := release
endif

RN :=
ifdef COMSPEC
RM := del
else
RM := rm -rf
endif

.PHONY: all fmt clean run debug docs cargo build

all: qemu

fmt:
	rustfmt --write-mode overwrite src/lib.rs

build:
	@cargo bootimage $(opt) --target $(target).json

qemu:
	@bootimage run $(opt) --target $(target).json || ([ $$? -eq 1 ] && exit 0) || exit 1

run: cargo
	@echo Run within ehyve
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

cargo:
	@echo Build for ehyve
	@cargo build -Z build-std=core,alloc --no-default-features $(opt) --target $(target).json

clean:
	@cargo clean

docs:
	@echo DOC
	@cargo doc
