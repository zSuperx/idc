section .text
global main
main:
.L7.prologue:
	push rbp
	mov rbp, rsp
	jmp .L0.entry
.L0.entry:
	; %0 -> rdi
	jmp .L1.body
.L1.body:
	jmp .L2.if
.L2.if:
	cmp rdi, 4
	mov %8, 1
	cmove %1, %8
	cmp %1, 1
	jz .L4.else
.L3.then:
	mov rax, 420
	jmp .L8.epilogue
.L4.else:
	mov %5, rdi
	imul %5, 2
	mov rax, %5
	jmp .L8.epilogue
.L8.epilogue:
	mov rsp, rbp
	pop rbp
	ret 
