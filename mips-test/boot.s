.global __start

__start:
    li $sp, 0x1000

    jal main
    nop

    b .
    nop

