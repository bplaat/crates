.section .text._start,"ax",@progbits
.global _start
.type _start,@function

_start:
    xor %ebp, %ebp
    mov (%rsp), %rdi
    lea 8(%rsp), %rsi
    lea 16(%rsp,%rdi,8), %rdx
    mov %rdx, environ(%rip)
    and $-16, %rsp
    call main
    mov %eax, %edi
    mov $231, %eax
    syscall
    hlt

.size _start, .-_start
.section .bss
.align 8
.global environ
.global __environ
environ:
__environ:
    .quad 0
.section .note.GNU-stack,"",@progbits
