default:
	make -C kernel
	make -C ehyve

clean:
	make -C kernel clean
	make -C ehyve clean

run:
	make -C kernel
	make -C ehyve
	make -C ehyve run

debug:
	make -C kernel debug
