section .multiboot_header

header_start:
	dd 0xE85250D6                ; integrity check
	dd 0                         ; protected mode switch
	dd header_end - header_start ; header length
	; yay checksum
	dd 0x100000000 - (0xE85250D6 + 0 + (header_end - header_start))

	; fun flags
	dw 0 ; type
	dw 0 ; flags
	dw 8 ; size
header_end:
