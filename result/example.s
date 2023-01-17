.intel_syntax
.global __lqd_main__
__lqd_main__:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
.__lqd_main__.L0:
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    push rax
    call main
    pop rax
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi

main:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
.main.L0:
    mov r13, 5000
    mov r15, r13
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    mov rdi, r15
    call square
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    mov r14, rax
    mov r15, r14
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    mov rdi, r15
    call cube
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    mov r15, rax

square:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
.square.L0:
    mov r13, rdi
    mov r14, rdi
    mov r15, r13
    imul r15, r14

cube:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
.cube.L0:
    mov r14, rdi
    mov r15, rdi
    mov r13, r14
    imul r13, r15
    mov r14, rdi
    mov r15, r13
    imul r15, r14

