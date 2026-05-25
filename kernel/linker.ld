/* 八卦 OS 内核链接脚本 — x86_64 */
ENTRY(start_32)

SECTIONS
{
    . = 1M;

    .multiboot2_header : ALIGN(8)
    {
        KEEP(*(.multiboot2_header))
    }

    .note.pvh : ALIGN(8)
    {
        KEEP(*(.note.pvh))
    }

    .text.boot :
    {
        *(.text.boot)
    }

    .text : ALIGN(4K)
    {
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
        *(.rodata.gdt)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .got : ALIGN(4K)
    {
        *(.got .got.*)
    }

    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    .bss.pagetables : ALIGN(4096)
    {
        *(.bss.pagetables)
    }

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.note.gnu.*)
        *(.comment)
    }
}
