# Minimal Multiboot2 kernel — assembly only
# Writes 'K' to QEMU debug port 0xE9 and halts

.section .multiboot2_header, "a"
.align 8
mb2_start:
    .long 0xe85250d6          # magic
    .long 0                   # architecture (x86)
    .long mb2_end - mb2_start # header length
    .long -(0xe85250d6 + 0 + (mb2_end - mb2_start))  # checksum
mb2_end:

.section .text
.globl _start
_start:
    movw $0xE9, %dx
    movb $'K', %al
    outb %al, %dx
    hlt
    jmp _start
