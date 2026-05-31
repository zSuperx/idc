section .text
global main
main:
	push rbp
	mov rbp, rsp
.F0.0.entry:
	; %0 -> rdi
	jmp .F0.1.body
.F0.1.body:
	jmp .F0.2.if
.F0.2.if:
	cmp rdi, 1
	mov %6, 1
	cmovg %1, %6
	cmp %1, 1
	jz .F0.4.else
.F0.3.then:
	mov rax, 69
	jmp .F0.7.x86exit
.F0.4.else:
	mov rax, 68
	jmp .F0.7.x86exit
.F0.7.x86exit:
	mov rsp, rbp
	pop rbp
	ret 
