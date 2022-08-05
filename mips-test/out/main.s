	.text
	.abicalls
	.option	pic0
	.section	.mdebug.abi32,"",@progbits
	.nan	legacy
	.text
	.file	"main.c"
	.file	0 "/home/antonio/Projects/psx-emu/mips-test" "main.c" md5 0x028634f99ecae7111bf47d632cd5375e
	.globl	factorial                       # -- Begin function factorial
	.p2align	2
	.type	factorial,@function
	.set	nomicromips
	.set	nomips16
	.ent	factorial
factorial:                              # @factorial
$tmp0:
.set $func_begin0, ($tmp0)
	.loc	0 2 0                           # main.c:2:0
	.cfi_sections .debug_frame
	.cfi_startproc
	.frame	$fp,40,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -40
	.cfi_def_cfa_offset 40
	sw	$ra, 36($sp)                    # 4-byte Folded Spill
	sw	$fp, 32($sp)                    # 4-byte Folded Spill
	.cfi_offset 31, -4
	.cfi_offset 30, -8
	move	$fp, $sp
	.cfi_def_cfa_register 30
	sw	$4, 24($fp)
$tmp1:
	.loc	0 3 9 prologue_end              # main.c:3:9
	lw	$1, 24($fp)
	nop
$tmp2:
	.loc	0 3 9 is_stmt 0                 # main.c:3:9
	bnez	$1, $BB0_3
	nop
# %bb.1:
	j	$BB0_2
	nop
$BB0_2:
	.loc	0 0 9                           # main.c:0:9
	addiu	$1, $zero, 1
$tmp3:
	.loc	0 3 17                          # main.c:3:17
	sw	$1, 28($fp)
	j	$BB0_4
	nop
$tmp4:
$BB0_3:
	.loc	0 5 12 is_stmt 1                # main.c:5:12
	lw	$1, 24($fp)
	nop
	sw	$1, 20($fp)                     # 4-byte Folded Spill
	.loc	0 5 28 is_stmt 0                # main.c:5:28
	addiu	$4, $1, -1
	.loc	0 5 16                          # main.c:5:16
	jal	factorial
	nop
	lw	$1, 20($fp)                     # 4-byte Folded Reload
	nop
	.loc	0 5 14                          # main.c:5:14
	mult	$1, $2
	mflo	$1
	.loc	0 5 5                           # main.c:5:5
	sw	$1, 28($fp)
	j	$BB0_4
	nop
$BB0_4:
	.loc	0 6 1 is_stmt 1                 # main.c:6:1
	lw	$2, 28($fp)
	move	$sp, $fp
	lw	$fp, 32($sp)                    # 4-byte Folded Reload
	lw	$ra, 36($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 40
	jr	$ra
	nop
$tmp5:
	.set	at
	.set	macro
	.set	reorder
	.end	factorial
$func_end0:
	.size	factorial, ($func_end0)-factorial
	.cfi_endproc
                                        # -- End function
	.globl	fibonnaci                       # -- Begin function fibonnaci
	.p2align	2
	.type	fibonnaci,@function
	.set	nomicromips
	.set	nomips16
	.ent	fibonnaci
fibonnaci:                              # @fibonnaci
$tmp6:
.set $func_begin1, ($tmp6)
	.loc	0 9 0                           # main.c:9:0
	.cfi_startproc
	.frame	$fp,40,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -40
	.cfi_def_cfa_offset 40
	sw	$ra, 36($sp)                    # 4-byte Folded Spill
	sw	$fp, 32($sp)                    # 4-byte Folded Spill
	.cfi_offset 31, -4
	.cfi_offset 30, -8
	move	$fp, $sp
	.cfi_def_cfa_register 30
	sw	$4, 24($fp)
$tmp7:
	.loc	0 10 9 prologue_end             # main.c:10:9
	lw	$1, 24($fp)
	addiu	$2, $zero, 1
$tmp8:
	.loc	0 10 9 is_stmt 0                # main.c:10:9
	bne	$1, $2, $BB1_3
	nop
# %bb.1:
	j	$BB1_2
	nop
$BB1_2:
	.loc	0 0 9                           # main.c:0:9
	addiu	$1, $zero, 1
$tmp9:
	.loc	0 10 17                         # main.c:10:17
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$tmp10:
$BB1_3:
	.loc	0 11 9 is_stmt 1                # main.c:11:9
	lw	$1, 24($fp)
	addiu	$2, $zero, 2
$tmp11:
	.loc	0 11 9 is_stmt 0                # main.c:11:9
	bne	$1, $2, $BB1_6
	nop
# %bb.4:
	j	$BB1_5
	nop
$BB1_5:
	.loc	0 0 9                           # main.c:0:9
	addiu	$1, $zero, 1
$tmp12:
	.loc	0 11 17                         # main.c:11:17
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$tmp13:
$BB1_6:
	.loc	0 13 22 is_stmt 1               # main.c:13:22
	lw	$1, 24($fp)
	nop
	.loc	0 13 24 is_stmt 0               # main.c:13:24
	addiu	$4, $1, -1
	.loc	0 13 12                         # main.c:13:12
	jal	fibonnaci
	nop
	sw	$2, 20($fp)                     # 4-byte Folded Spill
	.loc	0 13 41                         # main.c:13:41
	lw	$1, 24($fp)
	nop
	.loc	0 13 43                         # main.c:13:43
	addiu	$4, $1, -2
	.loc	0 13 31                         # main.c:13:31
	jal	fibonnaci
	nop
	move	$1, $2
	.loc	0 13 29                         # main.c:13:29
	lw	$2, 20($fp)                     # 4-byte Folded Reload
	nop
	addu	$1, $2, $1
	.loc	0 13 5                          # main.c:13:5
	sw	$1, 28($fp)
	j	$BB1_7
	nop
$BB1_7:
	.loc	0 14 1 is_stmt 1                # main.c:14:1
	lw	$2, 28($fp)
	move	$sp, $fp
	lw	$fp, 32($sp)                    # 4-byte Folded Reload
	lw	$ra, 36($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 40
	jr	$ra
	nop
$tmp14:
	.set	at
	.set	macro
	.set	reorder
	.end	fibonnaci
$func_end1:
	.size	fibonnaci, ($func_end1)-fibonnaci
	.cfi_endproc
                                        # -- End function
	.globl	print                           # -- Begin function print
	.p2align	2
	.type	print,@function
	.set	nomicromips
	.set	nomips16
	.ent	print
print:                                  # @print
$tmp15:
.set $func_begin2, ($tmp15)
	.loc	0 17 0                          # main.c:17:0
	.cfi_startproc
	.frame	$fp,16,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -16
	.cfi_def_cfa_offset 16
	sw	$ra, 12($sp)                    # 4-byte Folded Spill
	sw	$fp, 8($sp)                     # 4-byte Folded Spill
	.cfi_offset 31, -4
	.cfi_offset 30, -8
	move	$fp, $sp
	.cfi_def_cfa_register 30
	sw	$4, 4($fp)
	lui	$1, 8144
	ori	$1, $1, 1016
$tmp16:
	.loc	0 18 20 prologue_end            # main.c:18:20
	sw	$1, 0($fp)
	.loc	0 19 5                          # main.c:19:5
	j	$BB2_1
	nop
$BB2_1:                                 # =>This Inner Loop Header: Depth=1
	.loc	0 19 13 is_stmt 0               # main.c:19:13
	lw	$1, 4($fp)
	nop
	.loc	0 19 12                         # main.c:19:12
	lbu	$1, 0($1)
	nop
	.loc	0 19 5                          # main.c:19:5
	beqz	$1, $BB2_4
	nop
# %bb.2:                                #   in Loop: Header=BB2_1 Depth=1
	j	$BB2_3
	nop
$BB2_3:                                 #   in Loop: Header=BB2_1 Depth=1
$tmp17:
	.loc	0 20 21 is_stmt 1               # main.c:20:21
	lw	$1, 4($fp)
	nop
	.loc	0 20 20 is_stmt 0               # main.c:20:20
	lbu	$1, 0($1)
	.loc	0 20 10                         # main.c:20:10
	lw	$2, 0($fp)
	nop
	.loc	0 20 18                         # main.c:20:18
	sb	$1, 0($2)
	.loc	0 21 9 is_stmt 1                # main.c:21:9
	lw	$1, 4($fp)
	nop
	addiu	$1, $1, 1
	sw	$1, 4($fp)
$tmp18:
	.loc	0 19 5                          # main.c:19:5
	j	$BB2_1
	nop
$BB2_4:
	.loc	0 23 1                          # main.c:23:1
	move	$sp, $fp
	lw	$fp, 8($sp)                     # 4-byte Folded Reload
	lw	$ra, 12($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 16
	jr	$ra
	nop
$tmp19:
	.set	at
	.set	macro
	.set	reorder
	.end	print
$func_end2:
	.size	print, ($func_end2)-print
	.cfi_endproc
                                        # -- End function
	.globl	main                            # -- Begin function main
	.p2align	2
	.type	main,@function
	.set	nomicromips
	.set	nomips16
	.ent	main
main:                                   # @main
$tmp20:
.set $func_begin3, ($tmp20)
	.loc	0 26 0                          # main.c:26:0
	.cfi_startproc
	.frame	$fp,32,$ra
	.mask 	0xc0000000,-4
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	.set	noat
# %bb.0:
	addiu	$sp, $sp, -32
	.cfi_def_cfa_offset 32
	sw	$ra, 28($sp)                    # 4-byte Folded Spill
	sw	$fp, 24($sp)                    # 4-byte Folded Spill
	.cfi_offset 31, -4
	.cfi_offset 30, -8
	move	$fp, $sp
	.cfi_def_cfa_register 30
	addiu	$4, $zero, 5
$tmp21:
	.loc	0 27 13 prologue_end            # main.c:27:13
	jal	fibonnaci
	nop
	.loc	0 27 9 is_stmt 0                # main.c:27:9
	sw	$2, 20($fp)
	.loc	0 30 5 is_stmt 1                # main.c:30:5
	lui	$1, %hi($.str)
	addiu	$4, $1, %lo($.str)
	jal	print
	nop
	addiu	$2, $zero, 0
	.loc	0 32 5                          # main.c:32:5
	move	$sp, $fp
	lw	$fp, 24($sp)                    # 4-byte Folded Reload
	lw	$ra, 28($sp)                    # 4-byte Folded Reload
	addiu	$sp, $sp, 32
	jr	$ra
	nop
$tmp22:
	.set	at
	.set	macro
	.set	reorder
	.end	main
$func_end3:
	.size	main, ($func_end3)-main
	.cfi_endproc
                                        # -- End function
	.type	$.str,@object                   # @.str
	.section	.rodata.str1.1,"aMS",@progbits,1
$.str:
	.asciz	"Hello world!\r\n"
	.size	$.str, 15

	.section	.debug_abbrev,"",@0x7000001e
	.byte	1                               # Abbreviation Code
	.byte	17                              # DW_TAG_compile_unit
	.byte	1                               # DW_CHILDREN_yes
	.byte	37                              # DW_AT_producer
	.byte	37                              # DW_FORM_strx1
	.byte	19                              # DW_AT_language
	.byte	5                               # DW_FORM_data2
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	114                             # DW_AT_str_offsets_base
	.byte	23                              # DW_FORM_sec_offset
	.byte	16                              # DW_AT_stmt_list
	.byte	23                              # DW_FORM_sec_offset
	.byte	27                              # DW_AT_comp_dir
	.byte	37                              # DW_FORM_strx1
	.byte	17                              # DW_AT_low_pc
	.byte	27                              # DW_FORM_addrx
	.byte	18                              # DW_AT_high_pc
	.byte	6                               # DW_FORM_data4
	.byte	115                             # DW_AT_addr_base
	.byte	23                              # DW_FORM_sec_offset
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	2                               # Abbreviation Code
	.byte	52                              # DW_TAG_variable
	.byte	0                               # DW_CHILDREN_no
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	2                               # DW_AT_location
	.byte	24                              # DW_FORM_exprloc
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	3                               # Abbreviation Code
	.byte	1                               # DW_TAG_array_type
	.byte	1                               # DW_CHILDREN_yes
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	4                               # Abbreviation Code
	.byte	33                              # DW_TAG_subrange_type
	.byte	0                               # DW_CHILDREN_no
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	55                              # DW_AT_count
	.byte	11                              # DW_FORM_data1
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	5                               # Abbreviation Code
	.byte	36                              # DW_TAG_base_type
	.byte	0                               # DW_CHILDREN_no
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	62                              # DW_AT_encoding
	.byte	11                              # DW_FORM_data1
	.byte	11                              # DW_AT_byte_size
	.byte	11                              # DW_FORM_data1
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	6                               # Abbreviation Code
	.byte	36                              # DW_TAG_base_type
	.byte	0                               # DW_CHILDREN_no
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	11                              # DW_AT_byte_size
	.byte	11                              # DW_FORM_data1
	.byte	62                              # DW_AT_encoding
	.byte	11                              # DW_FORM_data1
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	7                               # Abbreviation Code
	.byte	15                              # DW_TAG_pointer_type
	.byte	0                               # DW_CHILDREN_no
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	8                               # Abbreviation Code
	.byte	53                              # DW_TAG_volatile_type
	.byte	0                               # DW_CHILDREN_no
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	9                               # Abbreviation Code
	.byte	46                              # DW_TAG_subprogram
	.byte	1                               # DW_CHILDREN_yes
	.byte	17                              # DW_AT_low_pc
	.byte	27                              # DW_FORM_addrx
	.byte	18                              # DW_AT_high_pc
	.byte	6                               # DW_FORM_data4
	.byte	64                              # DW_AT_frame_base
	.byte	24                              # DW_FORM_exprloc
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	39                              # DW_AT_prototyped
	.byte	25                              # DW_FORM_flag_present
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	63                              # DW_AT_external
	.byte	25                              # DW_FORM_flag_present
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	10                              # Abbreviation Code
	.byte	5                               # DW_TAG_formal_parameter
	.byte	0                               # DW_CHILDREN_no
	.byte	2                               # DW_AT_location
	.byte	24                              # DW_FORM_exprloc
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	11                              # Abbreviation Code
	.byte	46                              # DW_TAG_subprogram
	.byte	1                               # DW_CHILDREN_yes
	.byte	17                              # DW_AT_low_pc
	.byte	27                              # DW_FORM_addrx
	.byte	18                              # DW_AT_high_pc
	.byte	6                               # DW_FORM_data4
	.byte	64                              # DW_AT_frame_base
	.byte	24                              # DW_FORM_exprloc
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	39                              # DW_AT_prototyped
	.byte	25                              # DW_FORM_flag_present
	.byte	63                              # DW_AT_external
	.byte	25                              # DW_FORM_flag_present
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	12                              # Abbreviation Code
	.byte	52                              # DW_TAG_variable
	.byte	0                               # DW_CHILDREN_no
	.byte	2                               # DW_AT_location
	.byte	24                              # DW_FORM_exprloc
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	13                              # Abbreviation Code
	.byte	46                              # DW_TAG_subprogram
	.byte	1                               # DW_CHILDREN_yes
	.byte	17                              # DW_AT_low_pc
	.byte	27                              # DW_FORM_addrx
	.byte	18                              # DW_AT_high_pc
	.byte	6                               # DW_FORM_data4
	.byte	64                              # DW_AT_frame_base
	.byte	24                              # DW_FORM_exprloc
	.byte	3                               # DW_AT_name
	.byte	37                              # DW_FORM_strx1
	.byte	58                              # DW_AT_decl_file
	.byte	11                              # DW_FORM_data1
	.byte	59                              # DW_AT_decl_line
	.byte	11                              # DW_FORM_data1
	.byte	73                              # DW_AT_type
	.byte	19                              # DW_FORM_ref4
	.byte	63                              # DW_AT_external
	.byte	25                              # DW_FORM_flag_present
	.byte	0                               # EOM(1)
	.byte	0                               # EOM(2)
	.byte	0                               # EOM(3)
	.section	.debug_info,"",@0x7000001e
$cu_begin0:
	.4byte	($debug_info_end0)-($debug_info_start0) # Length of Unit
$debug_info_start0:
	.2byte	5                               # DWARF version number
	.byte	1                               # DWARF Unit Type
	.byte	4                               # Address Size (in bytes)
	.4byte	.debug_abbrev                   # Offset Into Abbrev. Section
	.byte	1                               # Abbrev [1] 0xc:0xbc DW_TAG_compile_unit
	.byte	0                               # DW_AT_producer
	.2byte	12                              # DW_AT_language
	.byte	1                               # DW_AT_name
	.4byte	($str_offsets_base0)            # DW_AT_str_offsets_base
	.4byte	($line_table_start0)            # DW_AT_stmt_list
	.byte	2                               # DW_AT_comp_dir
	.byte	1                               # DW_AT_low_pc
	.4byte	($func_end3)-($func_begin0)     # DW_AT_high_pc
	.4byte	($addr_table_base0)             # DW_AT_addr_base
	.byte	2                               # Abbrev [2] 0x23:0xa DW_TAG_variable
	.4byte	45                              # DW_AT_type
	.byte	0                               # DW_AT_decl_file
	.byte	30                              # DW_AT_decl_line
	.byte	2                               # DW_AT_location
	.byte	161
	.byte	0
	.byte	3                               # Abbrev [3] 0x2d:0xc DW_TAG_array_type
	.4byte	57                              # DW_AT_type
	.byte	4                               # Abbrev [4] 0x32:0x6 DW_TAG_subrange_type
	.4byte	61                              # DW_AT_type
	.byte	15                              # DW_AT_count
	.byte	0                               # End Of Children Mark
	.byte	5                               # Abbrev [5] 0x39:0x4 DW_TAG_base_type
	.byte	3                               # DW_AT_name
	.byte	6                               # DW_AT_encoding
	.byte	1                               # DW_AT_byte_size
	.byte	6                               # Abbrev [6] 0x3d:0x4 DW_TAG_base_type
	.byte	4                               # DW_AT_name
	.byte	8                               # DW_AT_byte_size
	.byte	7                               # DW_AT_encoding
	.byte	7                               # Abbrev [7] 0x41:0x5 DW_TAG_pointer_type
	.4byte	70                              # DW_AT_type
	.byte	8                               # Abbrev [8] 0x46:0x5 DW_TAG_volatile_type
	.4byte	57                              # DW_AT_type
	.byte	9                               # Abbrev [9] 0x4b:0x1b DW_TAG_subprogram
	.byte	1                               # DW_AT_low_pc
	.4byte	($func_end0)-($func_begin0)     # DW_AT_high_pc
	.byte	1                               # DW_AT_frame_base
	.byte	110
	.byte	5                               # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	1                               # DW_AT_decl_line
                                        # DW_AT_prototyped
	.4byte	190                             # DW_AT_type
                                        # DW_AT_external
	.byte	10                              # Abbrev [10] 0x5a:0xb DW_TAG_formal_parameter
	.byte	2                               # DW_AT_location
	.byte	141
	.byte	24
	.byte	10                              # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	1                               # DW_AT_decl_line
	.4byte	190                             # DW_AT_type
	.byte	0                               # End Of Children Mark
	.byte	9                               # Abbrev [9] 0x66:0x1b DW_TAG_subprogram
	.byte	2                               # DW_AT_low_pc
	.4byte	($func_end1)-($func_begin1)     # DW_AT_high_pc
	.byte	1                               # DW_AT_frame_base
	.byte	110
	.byte	7                               # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	8                               # DW_AT_decl_line
                                        # DW_AT_prototyped
	.4byte	190                             # DW_AT_type
                                        # DW_AT_external
	.byte	10                              # Abbrev [10] 0x75:0xb DW_TAG_formal_parameter
	.byte	2                               # DW_AT_location
	.byte	141
	.byte	24
	.byte	10                              # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	8                               # DW_AT_decl_line
	.4byte	190                             # DW_AT_type
	.byte	0                               # End Of Children Mark
	.byte	11                              # Abbrev [11] 0x81:0x22 DW_TAG_subprogram
	.byte	3                               # DW_AT_low_pc
	.4byte	($func_end2)-($func_begin2)     # DW_AT_high_pc
	.byte	1                               # DW_AT_frame_base
	.byte	110
	.byte	8                               # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	16                              # DW_AT_decl_line
                                        # DW_AT_prototyped
                                        # DW_AT_external
	.byte	10                              # Abbrev [10] 0x8c:0xb DW_TAG_formal_parameter
	.byte	2                               # DW_AT_location
	.byte	141
	.byte	4
	.byte	11                              # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	16                              # DW_AT_decl_line
	.4byte	194                             # DW_AT_type
	.byte	12                              # Abbrev [12] 0x97:0xb DW_TAG_variable
	.byte	2                               # DW_AT_location
	.byte	141
	.byte	0
	.byte	12                              # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	18                              # DW_AT_decl_line
	.4byte	65                              # DW_AT_type
	.byte	0                               # End Of Children Mark
	.byte	13                              # Abbrev [13] 0xa3:0x1b DW_TAG_subprogram
	.byte	4                               # DW_AT_low_pc
	.4byte	($func_end3)-($func_begin3)     # DW_AT_high_pc
	.byte	1                               # DW_AT_frame_base
	.byte	110
	.byte	9                               # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	25                              # DW_AT_decl_line
	.4byte	190                             # DW_AT_type
                                        # DW_AT_external
	.byte	12                              # Abbrev [12] 0xb2:0xb DW_TAG_variable
	.byte	2                               # DW_AT_location
	.byte	141
	.byte	20
	.byte	13                              # DW_AT_name
	.byte	0                               # DW_AT_decl_file
	.byte	27                              # DW_AT_decl_line
	.4byte	190                             # DW_AT_type
	.byte	0                               # End Of Children Mark
	.byte	5                               # Abbrev [5] 0xbe:0x4 DW_TAG_base_type
	.byte	6                               # DW_AT_name
	.byte	5                               # DW_AT_encoding
	.byte	4                               # DW_AT_byte_size
	.byte	7                               # Abbrev [7] 0xc2:0x5 DW_TAG_pointer_type
	.4byte	57                              # DW_AT_type
	.byte	0                               # End Of Children Mark
$debug_info_end0:
	.section	.debug_str_offsets,"",@0x7000001e
	.4byte	60                              # Length of String Offsets Set
	.2byte	5
	.2byte	0
$str_offsets_base0:
	.section	.debug_str,"MS",@0x7000001e,1
$info_string0:
	.asciz	"clang version 16.0.0 (git@github.com:antangelo/llvm-project.git c9e1ecd46da5c31cd7da3207691567906a100231)" # string offset=0
$info_string1:
	.asciz	"main.c"                        # string offset=106
$info_string2:
	.asciz	"/home/antonio/Projects/psx-emu/mips-test" # string offset=113
$info_string3:
	.asciz	"char"                          # string offset=154
$info_string4:
	.asciz	"__ARRAY_SIZE_TYPE__"           # string offset=159
$info_string5:
	.asciz	"factorial"                     # string offset=179
$info_string6:
	.asciz	"int"                           # string offset=189
$info_string7:
	.asciz	"fibonnaci"                     # string offset=193
$info_string8:
	.asciz	"print"                         # string offset=203
$info_string9:
	.asciz	"main"                          # string offset=209
$info_string10:
	.asciz	"n"                             # string offset=214
$info_string11:
	.asciz	"c"                             # string offset=216
$info_string12:
	.asciz	"printer"                       # string offset=218
$info_string13:
	.asciz	"f"                             # string offset=226
	.section	.debug_str_offsets,"",@0x7000001e
	.4byte	($info_string0)
	.4byte	($info_string1)
	.4byte	($info_string2)
	.4byte	($info_string3)
	.4byte	($info_string4)
	.4byte	($info_string5)
	.4byte	($info_string6)
	.4byte	($info_string7)
	.4byte	($info_string8)
	.4byte	($info_string9)
	.4byte	($info_string10)
	.4byte	($info_string11)
	.4byte	($info_string12)
	.4byte	($info_string13)
	.section	.debug_addr,"",@0x7000001e
	.4byte	($debug_addr_end0)-($debug_addr_start0) # Length of contribution
$debug_addr_start0:
	.2byte	5                               # DWARF version number
	.byte	4                               # Address size
	.byte	0                               # Segment selector size
$addr_table_base0:
	.4byte	($.str)
	.4byte	($func_begin0)
	.4byte	($func_begin1)
	.4byte	($func_begin2)
	.4byte	($func_begin3)
$debug_addr_end0:
	.ident	"clang version 16.0.0 (git@github.com:antangelo/llvm-project.git c9e1ecd46da5c31cd7da3207691567906a100231)"
	.section	".note.GNU-stack","",@progbits
	.addrsig
	.addrsig_sym factorial
	.addrsig_sym fibonnaci
	.addrsig_sym print
	.text
	.section	.debug_line,"",@0x7000001e
$line_table_start0:
