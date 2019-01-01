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

all: cargo

fmt:
	rustfmt --write-mode overwrite src/lib.rs

run:
	@ehyve target/$(arch)-eduos/$(rdir)/eduos-rs

clean:
	$(RM) target

docs:
	@echo DOC
	@cargo doc

cargo:
	@echo CARGO
	@cargo xbuild $(opt) --target $(target).json
