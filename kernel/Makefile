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

.PHONY: all fmt clean run debug cargo docs

all: cargo $(build_wasm)

fmt:
	rustfmt --write-mode overwrite src/lib.rs

clean:
	rm -rf target

eduos.wasm: cargo
	@echo WASM_GC
	@wasm-gc target/$(target)/$(rdir)/eduos_rs.wasm eduos.wasm

docs:
	@echo DOC
	@cargo doc

cargo:
	@echo CARGO
	@cargo xbuild $(opt) --target $(target).json
