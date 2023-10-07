    .section .text.entry
    .globl _start
_start:
    la sp, stack_top
    call rust_main

    .section .bss.stack
    .globl stack_lb
stack_lb:
    .space 4096 * 16
    .globl stack_top
stack_top: