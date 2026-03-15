/*
 * RuVix AArch64 boot — minimal entry for QEMU virt
 * _start → setup stack → clear BSS → kernel_main (Rust)
 */

.section .text.boot
.global _start

_start:
    /* Disable interrupts */
    msr daifset, #0xf

    /* Check exception level — drop to EL1 if at EL2 */
    mrs x0, CurrentEL
    and x0, x0, #0xC
    cmp x0, #8
    b.eq from_el2
    b setup_stack

from_el2:
    mov x0, #(1 << 31)
    msr hcr_el2, x0
    mov x0, #0x5
    msr spsr_el2, x0
    adr x0, setup_stack
    msr elr_el2, x0
    eret

setup_stack:
    adr x0, __stack_top
    mov sp, x0
    mov x29, xzr

    /* Clear BSS */
    adr x0, __bss_start
    adr x1, __bss_end
    sub x2, x1, x0
    cbz x2, 2f
1:
    str xzr, [x0], #8
    subs x2, x2, #8
    b.hi 1b
2:

    /* Jump to Rust */
    bl kernel_main

    /* Should never return */
3:
    wfi
    b 3b
