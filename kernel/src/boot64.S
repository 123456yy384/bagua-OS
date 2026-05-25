# 32→64 long mode trampoline for Multiboot2

.section .text.boot
.code32
.globl start_32
start_32:
    cli
    cld

    # Write '3' to serial to confirm 32-bit entry
    movw $0x3F8, %dx
    movb $0x33, %al; outb %al, %dx

    # Check CPUID extended function 0x80000001
    movl $0x80000000, %eax; cpuid
    cmpl $0x80000001, %eax; jb .Lno64

    movw $0x3F8, %dx; movb $0x41, %al; outb %al, %dx  # 'A' = CPUID OK

    movl $0x80000001, %eax; cpuid
    testl $0x20000000, %edx; jz .Lno64

    movw $0x3F8, %dx; movb $0x42, %al; outb %al, %dx  # 'B' = LM OK

    # Set up page tables
    movl $pdpt, %eax; orl $3, %eax; movl %eax, pml4
    movl $pd, %eax; orl $3, %eax; movl %eax, pdpt
    movl $0x83, %eax; xorl %ebx, %ebx; movl $pd, %ecx
1:  movl %ebx, (%ecx); movl $0, 4(%ecx); orl %eax, (%ecx)
    addl $8, %ecx; addl $0x200000, %ebx
    cmpl $pd_end, %ecx; jb 1b

    movw $0x3F8, %dx; movb $0x43, %al; outb %al, %dx  # 'C' = PT OK

    movl $pml4, %eax; movl %eax, %cr3
    movl %cr4, %eax; orl $0x20, %eax; movl %eax, %cr4

    movl $0xC0000080, %ecx; rdmsr; orl $0x100, %eax; wrmsr

    movw $0x3F8, %dx; movb $0x44, %al; outb %al, %dx  # 'D' = EFER OK

    movl %cr0, %eax; orl $0x80000000, %eax; movl %eax, %cr0

    movw $0x3F8, %dx; movb $0x45, %al; outb %al, %dx  # 'E' = Paging ON

    lgdt gdt64_ptr
    ljmpl $8, $_start64

.Lno64:
    movw $0x3F8, %dx
    movb $0x58, %al; outb %al, %dx  # 'X'
    hlt; jmp .Lno64

# ── 64-bit entry ──
.code64
_start64:
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss

    # Write "BAGUA" to serial COM1 for verification
    movw $0x3F8, %dx
    movb $0x42, %al; outb %al, %dx  # B
    movb $0x41, %al; outb %al, %dx  # A
    movb $0x47, %al; outb %al, %dx  # G
    movb $0x55, %al; outb %al, %dx  # U
    movb $0x41, %al; outb %al, %dx  # A
    movb $0x0A, %al; outb %al, %dx  # \n

    jmp _start

# ── 64-bit GDT ──
.section .rodata.gdt, "a"
.align 16
gdt64:
    .quad 0x0000000000000000   # null
    .quad 0x00209A0000000000   # 64-bit code (selector 0x08)
    .quad 0x0000920000000000   # data        (selector 0x10)
gdt64_end:
gdt64_ptr:
    .word gdt64_end - gdt64 - 1
    .quad gdt64

# ── Page tables (BSS, zeroed by bootloader) ──
.section .bss.pagetables
.align 4096
pml4: .skip 4096
pdpt: .skip 4096
pd:   .skip 4096
pd_end:
