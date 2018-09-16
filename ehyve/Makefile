release ?=

opt :=
rdir := debug

ifeq ($(release), 1)
opt := --release
rdir := release
endif

RM :=
ifdef COMSPEC
RM := del
else
RM := rm -rf
endif

.PHONY: all clean docs

all:
	@echo CARGO
	@cargo build $(opt)

clean:
	$(RM) target

docs:
	@echo DOC
	@cargo doc
