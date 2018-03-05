ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
        /* multiboot header at start */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }
}
