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

.PHONY: all build fmt clean run debug docs

run: build
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

build:
	@echo Build for ehyve
	@cargo build -Z build-std=core,alloc --no-default-features $(opt) --target $(target).json

fmt:
	rustfmt --write-mode overwrite src/lib.rs

clean:
	@cargo clean

docs:
	@echo DOC
	@cargo doc
