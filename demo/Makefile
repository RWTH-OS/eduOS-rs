default: hello.o
	musl-gcc -fPIE -pie -Wall -o hello hello.o

hello.o: hello.c
	musl-gcc -fPIE -pie -Wall -c -o hello.o hello.c

clean:
	rm -rf hello *.o
