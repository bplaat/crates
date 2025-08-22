.intel_syntax noprefix

#ifdef __APPLE__
.equ SYSCALL_WRITE, 0x2000004
.equ SYSCALL_EXIT, 0x2000001
#endif
#ifdef __linux__
.equ SYSCALL_WRITE, 1
.equ SYSCALL_EXIT, 60
#endif
.equ STDOUT, 1

.text

.global _start
_start:
    lea rdi, [rip + hello]
    call strlen
    mov edx, eax
    lea rsi, [rip + hello]
    mov edi, STDOUT
    mov eax, SYSCALL_WRITE
    syscall

    xor edi, edi
    mov eax, SYSCALL_EXIT
    syscall

strlen:
    mov rax, rdi
.repeat:
    cmp BYTE PTR [rax], 0
    je .done
    inc rax
    jmp .repeat
.done:
    sub rax, rdi
    ret

.data

hello:
    .asciz "Hello x86_64 assembler!\n"
