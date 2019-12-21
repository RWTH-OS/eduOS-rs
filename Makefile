arch ?= x86_64
target ?= $(arch)-eduos
release ?=

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

#ifeq ($(arch), x86_64)
#BUILD_COMMAD := cargo bootimage $(opt) --target $(target).json
#RUN_COMMAND := bootimage run $(opt) --target $(target).json || ([ $$? -eq 1 ] && exit 0) || exit 1
#else
#BUILD_COMMAD := cargo build -Z build-std=core,alloc --no-default-features $(opt) --target $(target).json
#RUN_COMMAND := qemu-system-aarch64 -semihosting -M virt -cpu cortex-a53 -m 1G -serial stdio -display none -kernel target/$(arch)-eduos/$(rdir)/eduos-rs || ([ $$? -eq 1 ] && exit 0) || exit 1
#endif

.PHONY: all build fmt clean run debug docs

all: run

fmt:
	rustfmt --write-mode overwrite src/lib.rs

build:
	@echo Build for ehyve
	@cargo build -Z build-std=core,alloc --no-default-features $(opt) --target $(target).json

run: build
	@echo Run within ehyve
	@ehyve -f ./demo/hello target/$(arch)-eduos/$(rdir)/eduos-rs

clean:
	@cargo clean

docs:
	@echo DOC
	@cargo doc
