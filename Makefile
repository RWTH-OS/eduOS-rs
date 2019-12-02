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

.PHONY: all fmt clean run debug cargo docs

all: qemu

build:
	@cargo bootimage $(opt) --target $(target).json

fmt:
	rustfmt --write-mode overwrite src/lib.rs

run: cargo
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

qemu: build
	@bootimage run $(opt) --target $(target).json || ([ $$? -eq 1 ] && exit 0) || exit 1

cargo:
	@echo Build for ehyve
	@cargo build -Z build-std=core,alloc --no-default-features $(opt) --target $(target).json

clean:
	@cargo clean

docs:
	@echo DOC
	@cargo doc
