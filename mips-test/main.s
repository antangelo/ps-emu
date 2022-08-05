	.text
	.abicalls
	.option	pic0
	.section	.mdebug.abi32,"",@progbits
	.nan	legacy
	.text
	.file	"main.c"
	.globl	factorial                       # -- Begin function factorial
	.p2align	2
	.type	factorial,@function
	.set	nomicromips
	.set	nomips16
	.ent	factorial
factorial:                              # @factorial
	.frame	$fp,40,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -40
	sw	$ra, 36($sp)                    # 4-byte Folded Spill
	sw	$fp, 32($sp)                    # 4-byte Folded Spill
	move	$fp, $sp
	sw	$4, 24($fp)
	lw	$1, 24($fp)
	bnez	$1, $BB0_3
	nop
# %bb.1:
	j	$BB0_2
	nop
$BB0_2:
	addiu	$1, $zero, 1
	sw	$1, 28($fp)
	j	$BB0_4
	nop
$BB0_3:
	lw	$1, 24($fp)
	sw	$1, 20($fp)                     # 4-byte Folded Spill
	addiu	$4, $1, -1
	jal	factorial
	nop
	lw	$1, 20($fp)                     # 4-byte Folded Reload
	mul	$1, $1, $2
	sw	$1, 28($fp)
	j	$BB0_4
	nop
$BB0_4:
	lw	$2, 28($fp)
	move	$sp, $fp
	lw	$fp, 32($sp)                    # 4-byte Folded Reload
	lw	$ra, 36($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 40
	jr	$ra
	nop
	.set	at
	.set	macro
	.set	reorder
	.end	factorial
$func_end0:
	.size	factorial, ($func_end0)-factorial
                                        # -- End function
	.globl	fibonnaci                       # -- Begin function fibonnaci
	.p2align	2
	.type	fibonnaci,@function
	.set	nomicromips
	.set	nomips16
	.ent	fibonnaci
fibonnaci:                              # @fibonnaci
	.frame	$fp,40,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -40
	sw	$ra, 36($sp)                    # 4-byte Folded Spill
	sw	$fp, 32($sp)                    # 4-byte Folded Spill
	move	$fp, $sp
	sw	$4, 24($fp)
	lw	$1, 24($fp)
	addiu	$2, $zero, 1
	bne	$1, $2, $BB1_3
	nop
# %bb.1:
	j	$BB1_2
	nop
$BB1_2:
	addiu	$1, $zero, 1
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$BB1_3:
	lw	$1, 24($fp)
	addiu	$2, $zero, 2
	bne	$1, $2, $BB1_6
	nop
# %bb.4:
	j	$BB1_5
	nop
$BB1_5:
	addiu	$1, $zero, 1
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$BB1_6:
	lw	$1, 24($fp)
	addiu	$4, $1, -1
	jal	fibonnaci
	nop
	sw	$2, 20($fp)                     # 4-byte Folded Spill
	lw	$1, 24($fp)
	addiu	$4, $1, -2
	jal	fibonnaci
	nop
	move	$1, $2
	lw	$2, 20($fp)                     # 4-byte Folded Reload
	addu	$1, $2, $1
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$BB1_7:
	lw	$2, 28($fp)
	move	$sp, $fp
	lw	$fp, 32($sp)                    # 4-byte Folded Reload
	lw	$ra, 36($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 40
	jr	$ra
	nop
	.set	at
	.set	macro
	.set	reorder
	.end	fibonnaci
$func_end1:
	.size	fibonnaci, ($func_end1)-fibonnaci
                                        # -- End function
	.globl	main                            # -- Begin function main
	.p2align	2
	.type	main,@function
	.set	nomicromips
	.set	nomips16
	.ent	main
main:                                   # @main
	.frame	$fp,40,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -40
	sw	$ra, 36($sp)                    # 4-byte Folded Spill
	sw	$fp, 32($sp)                    # 4-byte Folded Spill
	move	$fp, $sp
	addiu	$4, $zero, 5
	jal	fibonnaci
	nop
	sw	$2, 24($fp)
	lui	$1, 65535
	ori	$1, $1, 12
	sw	$1, 20($fp)
	lw	$1, 24($fp)
	lw	$2, 20($fp)
	sw	$1, 0($2)
	lw	$2, 28($fp)
	move	$sp, $fp
	lw	$fp, 32($sp)                    # 4-byte Folded Reload
	lw	$ra, 36($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 40
	jr	$ra
	nop
	.set	at
	.set	macro
	.set	reorder
	.end	main
$func_end2:
	.size	main, ($func_end2)-main
                                        # -- End function
	.ident	"clang version 14.0.6"
	.section	".note.GNU-stack","",@progbits
	.addrsig
	.addrsig_sym factorial
	.addrsig_sym fibonnaci
	.text
