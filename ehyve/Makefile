release ?=

opt :=
rdir := debug

ifeq ($(release), 1)
opt := --release
rdir := release
endif

.PHONY: all clean docs

all:
	@echo CARGO
	@cargo build $(opt)

run:
	@./target/$(rdir)/ehyve ../kernel/build/kernel-x86_64.elf

clean:
	rm -rf build target

docs:
	@echo DOC
	@cargo doc
