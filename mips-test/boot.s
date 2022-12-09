.global __start
.global waste_time
.global reg_fib

waste_time:
    lui $2, 0xffff
    ori $2, $2, 0xffff
wt_loop:
    addiu $2, $2, -1
    bne $2, $0, wt_loop
    nop
    jr $ra
    nop

reg_fib:
    ori $a1, $0, 1
    ori $a2, $0, 1
    addiu $a0, $a0, -2
rf_loop:
    move $a3, $a2
    move $a2, $a1
    add $a1, $a1, $a3
    addiu $a0, $a0, -1
    bne $a0, $0, rf_loop
    nop
    move $v0, $a1
    jr $ra
    nop

__start:
    addiu $sp, $0, 0
    lui $sp, 0x01f0

    jal main
    nop

    b .
    nop

