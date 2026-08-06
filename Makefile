default:
	echo hi read the Makefile

out:
	mkdir -p out/

runtime/rt.o: out runtime/rt.s
	@echo Creating Runtime...
	nasm -felf64 runtime/rt.s -o runtime/rt.o

out/%.s: out tests/%.idc
	@echo Compiling...
	cargo r -- tests/$*.idc -S -o out/$*.s

out/%.o: out out/%.s
	@echo Assembling...
	nasm -felf64 out/$*.s -o out/$*.o

out/%: out runtime/rt.o out/%.o 
	@echo Linking...
	mold runtime/rt.o out/$*.o -o out/$*


.PHONY: clean
clean:
	rm -rf out/

