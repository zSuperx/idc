TESTS := ./tests
OUT_DIR := $(TESTS)/out

default:
	@echo hi read the Makefile

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

runtime/rt.o: $(OUT_DIR) runtime/rt.s
	@echo Creating Runtime...
	nasm -felf64 runtime/rt.s -o runtime/rt.o

$(OUT_DIR)/%.s: $(OUT_DIR) $(TESTS)/%.st
	@echo Compiling...
	cargo r -- $(TESTS)/$*.st -S -o $(OUT_DIR)/$*.s

$(OUT_DIR)/%.o: $(OUT_DIR) $(OUT_DIR)/%.s
	@echo Assembling...
	nasm -felf64 $(OUT_DIR)/$*.s -o $(OUT_DIR)/$*.o

$(OUT_DIR)/%: $(OUT_DIR) runtime/rt.o $(OUT_DIR)/%.o 
	@echo Linking...
	ld runtime/rt.o $(OUT_DIR)/$*.o -o $(OUT_DIR)/$*


.PHONY: clean
clean:
	rm -rf $(OUT_DIR)/

