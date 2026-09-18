.section .text._start,"ax",@progbits
.global _start
.type _start,%function

_start:
    mov x29, xzr
    ldr x0, [sp]
    add x1, sp, #8
    add x2, x1, x0, lsl #3
    add x2, x2, #8
    adrp x3, environ
    add x3, x3, :lo12:environ
    str x2, [x3]
    bl main
    mov x8, #94
    svc #0
    brk #0

.size _start, .-_start
.section .bss
.align 3
.global environ
.global __environ
environ:
__environ:
    .xword 0
.section .note.GNU-stack,"",%progbits
