# Withered makefile
# Copyright 2018 AnonymousDapper

default: build

.PHONY: default build run clean cargo

target/multiboot_header.o: src/asm/multiboot_header.asm
	mkdir -p target
	nasm -f elf64 src/asm/multiboot_header.asm -o target/multiboot_header.o

target/boot.o: src/asm/boot.asm
	mkdir -p target
	nasm -f elf64 src/asm/boot.asm -o target/boot.o

target/kernel.bin: target/multiboot_header.o target/boot.o src/asm/linker.ld cargo
	ld -n -o target/kernel.bin -T src/asm/linker.ld target/multiboot_header.o target/boot.o target/x86_64-unknown-withered-gnu/release/libwithered.a

target/os.iso: target/kernel.bin src/asm/grub.cfg
	mkdir -p target/isos/boot/grub
	cp src/asm/grub.cfg target/isos/boot/grub/
	cp target/kernel.bin target/isos/boot/
	grub-mkrescue -o target/os.iso target/isos

clean:
	cargo clean

build: target/kernel.bin # dumb semantic hack

cargo:
	xargo build --release --target x86_64-unknown-withered-gnu

run: target/os.iso
	qemu-system-x86_64 -cdrom target/os.iso
