default:
	echo hi read the Makefile

runtime/rt.o: runtime/rt.s
	nasm -felf64 runtime/rt.s -o runtime/rt.o

%.s: %.idc
	cargo r -- $*.idc -S

%.o: %.s
	nasm -felf64 $*.s -o $*.o

examples/%: runtime/rt.o examples/%.o
	mold runtime/rt.o examples/$*.o -o examples/$*


.PHONY: clean
clean:
	cargo clean
	rm examples/*.s examples/*.o examples/*.ir
