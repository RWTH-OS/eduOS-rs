; This file is dual licensed under the MIT and the University of Illinois Open
; Source Licenses. See LICENSE.TXT for details.
;
; du_int __umoddi3(du_int a, du_int b);
;
; result = remainder of a / b.
; both inputs and the output are 64-bit unsigned integers.
; This will do whatever the underlying hardware is set to do on division by zero.
; No other exceptions are generated, as the divide cannot overflow.
;
; This is targeted at 32-bit x86 *only*, as this can be done directly in hardware
; on x86_64.  The performance goal is ~40 cycles per divide, which is faster than
; currently possible via simulation of integer divides on the x87 unit.
;
;
; Stephen Canon, December 2008

; converted to NASM by Stefan Lankes, November 2017
; original code is taken from https://github.com/llvm-mirror/compiler-rt/

%ifidn __OUTPUT_FORMAT__, elf32

section .text
align 4
global __umoddi3
__umoddi3:
	push ebx
	mov [esp+20], ebx ; Find the index i of the leading bit in b.
	bsr ecx, ebx      ; If the high word of b is zero, jump to
	jz L9             ; the code to handle that special case [9].

	; High word of b is known to be non-zero on this branch

	mov eax, [esp+16] ; Construct bhi, containing bits [1+i:32+i] of b

	shr eax, cl       ; Practically, this means that bhi is given by:
	shr eax, 1
	not ecx           ; bhi = (high word of b) << (31 - i) |
	shl ebx, cl       ; (low word of b) >> (1 + i)
	or ebx, eax
	mov edx, [esp+12] ; Load the high and low words of a, and jump
	mov eax, [esp+8]  ; to [2] if the high word is larger than bhi
	cmp edx, ebx      ; to avoid overflowing the upcoming divide.
	jae L2

	; High word of a is greater than or equal to (b >> (1 + i)) on this branch

	div ebx           ; eax <-- qs, edx <-- r such that ahi:alo = bs*qs + r

	push edi
	not ecx
	shr eax, 1
	shr eax, cl       ; q = qs >> (1 + i)
	mov edi, eax
	mul	DWORD [esp+20]     ; q*blo
	mov ebx, [esp+12]
	mov ecx, [esp+16] ; ECX:EBX = a
	sub ebx, eax
	sbb ecx, edx      ; ECX:EBX = a - q*blo
	mov eax, [esp+24]
	imul eax, edi     ; q*bhi
	sub ecx, eax      ; ECX:EBX = a - q*b

	jnc L1            ; if positive, this is the result.
	add ebx, [esp+20] ; otherwise
	adc ecx, [esp+24] ; ECX:EBX = a - (q-1)*b = result
L1:	mov eax, ebx
	mov edx, ecx

	pop edi
	pop ebx
	ret


L2:	; High word of a is greater than or equal to (b >> (1 + i)) on this branch

	sub edx, ebx                        ; subtract bhi from ahi so that divide will not
	div ebx                             ; overflow, and find q and r such that
                                        ;
                                        ; ahi:alo = (1:q)*bhi + r
										;
										; Note that q is a number in (31-i).(1+i)
										; fix point.

	push edi
	not ecx
	shr eax, 1
	or eax, 0x80000000
	shr eax, cl                         ; q = (1:qs) >> (1 + i)
	mov edi, eax
	mul DWORD [esp+20]                        ; q*blo
	mov ebx, [esp+12]
	mov ecx, [esp+16]                   ; ECX:EBX = a
	sub ebx, eax
	sbb ecx, edx                        ;	 ECX:EBX = a - q*blo
	mov eax, [esp+24]
	imul eax, edi                       ; q*bhi
	sub ecx, eax                        ; ECX:EBX = a - q*b

	jnc L3                              ; if positive, this is the result.
	add ebx, [esp+20]                   ; otherwise
	adc ecx, [esp+24]                   ; ECX:EBX = a - (q-1)*b = result
L3:	mov eax, ebx
	mov edx, ecx

	pop edi
	pop ebx
	ret


L9:	; High word of b is zero on this branch

	mov eax, [esp+12]                   ; Find qhi and rhi such that
	mov ecx, [esp+16]
	xor edx, edx                        ; ahi = qhi*b + rhi	with	0 ≤ rhi < b
	div ecx
	mov ebx, eax
	mov eax, [esp+8]                    ; Find rlo such that
	div ecx
	mov eax, edx                        ; rhi:alo = qlo*b + rlo  with 0 ≤ rlo < b
	pop ebx
	xor edx, edx                        ; and return 0:rlo
	ret
%endif
