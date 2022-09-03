.global __start

__start:
    addiu $sp, $0, 0
    lui $sp, 0x01f0

    jal main
    nop

    b .
    nop

