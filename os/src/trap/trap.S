.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .align 2
__trap_entry:
    csrrw sp, sscratch, sp
    sd x1, 8(sp)
    sd x3, 24(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    sd t2, 16(sp)
    # satp
    ld t0, 34*8(sp)
    # trap_handler
    ld t1, 36*8(sp)
    # kernel sp
    ld sp, 35*8(sp)
    # switch to kernel sapce
    csrw satp, t0
    sfence.vma
    jr t1

__restore:
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    ld x1, 8(sp)
    ld x3, 24(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 16(sp)
    sret