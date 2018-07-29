release ?=

opt :=

ifeq ($(release), 1)
opt := --release
endif

.PHONY: all clean docs

all:
	@echo CARGO
	@cargo build $(opt)

clean:
	rm -rf build target

docs:
	@echo DOC
	@cargo doc
