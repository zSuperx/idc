default:
	echo hi read the Makefile

out:
	mkdir -p out/

runtime/rt.o: out runtime/rt.s
	nasm -felf64 runtime/rt.s -o runtime/rt.o

out/%.s: out tests/%.idc
	cargo r -- tests/$*.idc -S -o out/$*.s

out/%.o: out out/%.s
	nasm -felf64 out/$*.s -o out/$*.o

out/%: out runtime/rt.o out/%.o 
	mold runtime/rt.o out/$*.o -o out/$*


.PHONY: clean
clean:
	rm -rf out/

