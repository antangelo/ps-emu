LLVM_DIR=../../llvm-project/build/bin
CLANG_TARGET=mipsel-none-eabi
CC = $(LLVM_DIR)/clang -target $(CLANG_TARGET) -S
AS = $(LLVM_DIR)/clang -target $(CLANG_TARGET) -c
LD = ld.lld

CFLAGS = -mcpu=mips1 -ffreestanding -nostdlib -nostdinc -g -O3
LDFLAGS = -static -nmagic

OBJS = out/main.o out/boot.o

all: main.elf

clean:
	-rm -r out
	-rm main.elf

out/main.s: main.c
	@mkdir -p out
	$(CC) $(CFLAGS) -o $@ $<

out/boot.o: boot.s
	@mkdir -p out
	$(AS) -o $@ $<

out/%.o: out/%.s
	@mkdir -p out
	$(AS) -o $@ $<

main.elf: $(OBJS)
	$(LD) $(LDFLAGS) -o $@ $(OBJS)

