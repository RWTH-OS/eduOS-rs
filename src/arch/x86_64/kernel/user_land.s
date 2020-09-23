// Copyright (c) 2020 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

// definition from consts.rs
// pub const USER_ENTRY: usize = 0x8000000000usize;

.section .text
.global jump_to_user_land

.align 16
jump_to_user_land:
    movabsq $0x8000000000, %rax
    and $0xfff, %rdi
    or %rax, %rdi
    mov $0x23, %ecx
    mov $0x2b,%edx
    swapgs
    mov %rcx, %ds
    mov %rcx, %es
    pushq %rcx
    movabsq $0x8000400000, %rax
    pushq %rax
    pushfq
    pushq %rdx
    pushq %rdi
    iretq
L1: jmp L1