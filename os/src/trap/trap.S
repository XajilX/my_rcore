.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

.align 2
__trap_entry:
    csrrw sp, sscratch, sp
    addi sp, sp, -34*8
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
    mv a0, sp
    call trap_handler

__restore:
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 16(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    ld x1, 8(sp)
    ld x3, 24(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    addi sp, sp, 34*8
    csrrw sp, sscratch, sp
    sret