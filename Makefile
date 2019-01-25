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

.PHONY: all clean run debug cargo docs demo

all: cargo

run:
	@ehyve --file demo/hello target/$(arch)-eduos/$(rdir)/eduos-rs

clean:
	$(RM) target

docs:
	@echo DOC
	@mv .cargo cargo
	@cargo doc
	@mv cargo .cargo

cargo:
	@echo CARGO
	@cargo xbuild $(opt) --target $(target).json
