default:
	echo hi read the Makefile

runtime/rt.o: runtime/rt.s
	nasm -felf64 runtime/rt.s -o runtime/rt.o

%.s: %.wc
	cargo r -- $*.zl -c

%.o: %.s
	nasm -felf64 $*.s -o $*.o

%: runtime/rt.o %.o
	mold $*.o runtime/rt.o -o $*


.PHONY: clean
clean:
	cargo clean
	rm examples/*.s examples/*.o examples/*.ir
