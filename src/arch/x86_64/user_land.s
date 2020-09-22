// Copyright (c) 2020 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

.section .text
.global jump_to_user_land

.align 16
jump_to_user_land:
    movw 0x23, %ds
    movw 0x23, %es
    pushq $0x23
    pushq %rsp
    addq $16, (%rsp)
    pushfq
    pushq $0x2b
    pushq %rdi
    iretq
L1: jmp L1