#ifdef __x86_64__
.intel_syntax noprefix
#endif

#ifdef __APPLE__
#ifdef __x86_64__
.equ SYSCALL_WRITE, 0x2000004
.equ SYSCALL_EXIT, 0x2000001
#endif
#ifdef __aarch64__
.equ SYSCALL_WRITE, 4
.equ SYSCALL_EXIT, 1
#endif
#endif

#ifdef __linux__
#ifdef __x86_64__
.equ SYSCALL_WRITE, 1
.equ SYSCALL_EXIT, 60
#endif
#ifdef __aarch64__
.equ SYSCALL_WRITE, 64
.equ SYSCALL_EXIT, 93
#endif
#endif

.equ STDOUT, 1

.text

#ifdef __x86_64__

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

#endif

#ifdef __aarch64__

.macro lea reg, symbol
#ifdef __APPLE__
    adrp \reg, \symbol@PAGE
    add \reg, \reg, \symbol@PAGEOFF
#endif
#ifdef __linux__
    adrp \reg, \symbol
    add \reg, \reg, :lo12:\symbol
#endif
.endm

.macro syscall num
#ifdef __APPLE__
    mov x16, \num
    svc 0
#endif
#ifdef __linux__
    mov x8, \num
    svc 0
#endif
.endm

.global _start
_start:
    lea x0, hello
    bl strlen
    mov x2, x0
    lea x1, hello
    mov x0, STDOUT
    syscall SYSCALL_WRITE

    mov x0, 0
    syscall SYSCALL_EXIT

strlen:
    mov x1, x0
.repeat:
    ldrb w2, [x1]
    cbz w2, .done
    add x1, x1, 1
    b .repeat
.done:
    sub x0, x1, x0
    ret

#endif

.data

hello:
#ifdef __x86_64__
    .asciz "Hello from x86_64 assembler!\n"
#endif
#ifdef __aarch64__
    .asciz "Hello from ARM64 assembler!\n"
#endif
