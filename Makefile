default:
	echo hi read the Makefile

runtime/rt.o: runtime/rt.s
	nasm -felf64 runtime/rt.s -o runtime/rt.o

%.s: %.wc
	cargo r -- $*.wc -c

%.o: %.s
	nasm -felf64 $*.s -o $*.o

%: runtime/rt.o %.o
	ld $*.o runtime/rt.o -o $*

