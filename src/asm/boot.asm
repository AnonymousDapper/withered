global start

extern kmain

section .text
bits 32

error:
	mov word [0xB8000], 0x0C45
	mov word [0xB8002], 0x0C52
	mov word [0xB8004], 0x0C52
	mov word [0xB8006], 0x0C3A
	mov word [0xB8008], 0x0C20
	mov byte [0xB800a], al
	hlt

start:
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

section .bss

align 4096

p4_table:
	resb 4096

p3_table:
	resb 4096

p2_table:
	resb 4096

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

section .text
bits 64

; long_mode_start:
; 	; long mode init is done, lets print a message
; 	mov dword [0xB8000], 0x0E200E3C ; '< '
; 	mov dword [0xB8004], 0x0E490E57 ; 'WI'
; 	mov dword [0xB8008], 0x0E480E54 ; 'TH'
; 	mov dword [0xB800C], 0x0E520E45 ; 'ER'
; 	mov dword [0xB8010], 0x0E440E45 ; 'ED'
; 	mov dword [0xB8014], 0x0E3E0E20 ; ' >'

; 	mov dword [0xB8018], 0x0E200E20
; 	mov dword [0xB801C], 0x0E200E20
; 	mov dword [0xB8020], 0x0E200E20
; 	mov dword [0xB8024], 0x0E200E20
; 	hlt


