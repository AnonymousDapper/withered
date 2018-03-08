; MIT License

; Copyright (c) 2018 AnonymousDapper

; Permission is hereby granted, free of charge, to any person obtaining a copy
; of this software and associated documentation files (the "Software"), to deal
; in the Software without restriction, including without limitation the rights
; to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
; copies of the Software, and to permit persons to whom the Software is
; furnished to do so, subject to the following conditions:

; The above copyright notice and this permission notice shall be included in all
; copies or substantial portions of the Software.

; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
; IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
; FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
; AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
; LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
; OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
; SOFTWARE.

global start

extern kmain

section .text
bits 32

start:
	; move multiboot pointer to edi
	mov esp, stack_top
	mov edi, ebx

	; init page tables
	mov eax, p3_table
	or eax, 0b11
	mov dword [p4_table + 0], eax
	; copy p3 into eax, set first two bits, copy back to p4

	; copy p2 into eax, set bits and copy to p3
	mov eax, p2_table
	or eax, 0b11
	mov dword [p3_table + 0], eax

	mov ecx, 0

	.map_p2_table: ; map p2 table with 1 GiB of addresses
		mov eax, 0x200000 ; 2MiB
		mul ecx
		or eax, 0b10000011 ; huge page bit at back
		mov [p2_table + ecx * 8], eax
		inc ecx
		cmp ecx, 512
		jne .map_p2_table

	mov eax, p4_table
	or eax, 0b11
	mov [p4_table + 511 * 8] ,eax
	; setting up recursive mapping for P4

	mov eax, p4_table
	mov cr3, eax
	; moving page table to CR3

	mov eax, cr4
	or eax, 1 << 5 ; 100000
	mov cr4, eax
	; enable PAE

	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr
	; transition to long mode ready

	mov eax, cr0
	or eax, 1 << 31
	or eax, 1 << 16
	mov cr0, eax
	; enable paging

	lgdt [gdt64.pointer] ; load GDT

	mov ax, gdt64.data
	mov ss, ax
	mov ds, ax
	mov es, ax
	; update segment registers

	; long mode time
	jmp gdt64.code:kmain

	hlt ; if we get to this, there's a problem

section .bss

align 4096

p4_table:
	resb 4096

p3_table:
	resb 4096

p2_table:
	resb 4096

stack_bottom:
	resb 4096 * 4

stack_top:

section .rodata
gdt64:
	dq 0

.code: equ $ - gdt64
	dq (1 << 44) | (1 << 47) | ( 1 << 41) | (1 << 43) | (1 << 53)

.data: equ $ - gdt64
	dq (1 << 44) | (1 << 47) | (1 << 41)

.pointer:
	dw .pointer - gdt64 - 1
	dq gdt64

