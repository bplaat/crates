#ifdef __x86_64__
.intel_syntax noprefix
#endif

.text

#ifdef __x86_64__
.global _add
_add:
    mov eax, edi
    add eax, esi
    ret
#endif

#ifdef __aarch64__
.global _add
_add:
    add w0, w0, w1
    ret
#endif
