release ?=

opt :=
rdir := debug

ifeq ($(release), 1)
opt := --release
rdir := release
endif

ifdef COMSPEC
RM ?= del
else
RM ?= rm -rf
endif

.PHONY: all clean docs

all:
	@echo CARGO
	@cargo build $(opt)

run:
	@./target/$(rdir)/ehyve ../kernel/build/kernel-x86_64.elf

clean:
	$(RM) build target

docs:
	@echo DOC
	@cargo doc
