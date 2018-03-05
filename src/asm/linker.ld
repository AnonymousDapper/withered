ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
        /* multiboot header at start */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }
}
