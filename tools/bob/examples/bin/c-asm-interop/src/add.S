#ifdef __x86_64__
.intel_syntax noprefix
#endif

.text

#ifdef __APPLE__
.global _add
_add:
#else
.global add
add:
#endif
#ifdef __x86_64__
    mov eax, edi
    add eax, esi
    ret
#endif
#ifdef __aarch64__
    add w0, w0, w1
    ret
#endif
