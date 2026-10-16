// A simple first-fit heap that grows into the libogc MEM1 and MEM2 arenas
#include <malloc.h>
#include <stdint.h>
#include <string.h>

#include "libogc.h"

#define GROW_SIZE (256 * 1024)

// Free chunks form a linked list sorted by address
typedef struct Chunk {
    size_t size;
    struct Chunk* next;
} Chunk;

// Stored just before every pointer we hand out
typedef struct {
    uintptr_t start;
    size_t size;
} Header;

static Chunk* free_list = NULL;

static uintptr_t align_up(uintptr_t value, size_t alignment) {
    return (value + alignment - 1) & ~(uintptr_t)(alignment - 1);
}

static void insert_chunk(uintptr_t start, size_t size) {
    Chunk** link = &free_list;
    while (*link && (uintptr_t)*link < start)
        link = &(*link)->next;

    Chunk* chunk = (Chunk*)start;
    chunk->size = size;
    chunk->next = *link;
    if (chunk->next && start + size == (uintptr_t)chunk->next) {
        chunk->size += chunk->next->size;
        chunk->next = chunk->next->next;
    }
    *link = chunk;

    // Merge with the previous chunk when they touch
    if (link != &free_list) {
        Chunk* previous = (Chunk*)((uintptr_t)link - offsetof(Chunk, next));
        if ((uintptr_t)previous + previous->size == start) {
            previous->size += chunk->size;
            previous->next = chunk->next;
        }
    }
}

static bool grow(size_t size) {
    size = align_up(size < GROW_SIZE ? GROW_SIZE : size, 32);
    uintptr_t start = align_up((uintptr_t)SYS_GetArena1Lo(), 32);
    if (start + size <= (uintptr_t)SYS_GetArena1Hi()) {
        SYS_SetArena1Lo((void*)(start + size));
    } else {
        start = align_up((uintptr_t)SYS_GetArena2Lo(), 32);
        if (start + size > (uintptr_t)SYS_GetArena2Hi())
            return false;
        SYS_SetArena2Lo((void*)(start + size));
    }
    insert_chunk(start, size);
    return true;
}

static void* allocate(size_t alignment, size_t size) {
    // Bigger than all memory, also keeps the size calculations below from overflowing
    if (size > 0x10000000 || alignment > 0x10000000 || (alignment & (alignment - 1)) != 0)
        return NULL;
    if (alignment < 16)
        alignment = 16;
    size = align_up(size, 16);
    for (int attempt = 0; attempt < 2; attempt++) {
        for (Chunk** link = &free_list; *link; link = &(*link)->next) {
            uintptr_t start = (uintptr_t)*link;
            size_t chunk_size = (*link)->size;
            uintptr_t pointer = align_up(start + sizeof(Header), alignment);
            uintptr_t end = pointer + size;
            if (end > start + chunk_size)
                continue;

            // Keep the leftover as a free chunk when it is big enough
            *link = (*link)->next;
            uintptr_t chunk_end = start + chunk_size;
            if (chunk_end - end >= 64)
                insert_chunk(end, chunk_end - end);
            else
                end = chunk_end;

            Header* header = (Header*)(pointer - sizeof(Header));
            header->start = start;
            header->size = end - start;
            return (void*)pointer;
        }
        if (!grow(size + alignment + sizeof(Header)))
            return NULL;
    }
    return NULL;
}

void* memalign(size_t alignment, size_t size) {
    uint32_t level = IRQ_Disable();
    void* pointer = allocate(alignment, size);
    IRQ_Restore(level);
    return pointer;
}

void* malloc(size_t size) {
    return memalign(16, size);
}

void* calloc(size_t count, size_t size) {
    if (size != 0 && count > SIZE_MAX / size)
        return NULL;
    void* pointer = malloc(count * size);
    if (pointer)
        memset(pointer, 0, count * size);
    return pointer;
}

void free(void* pointer) {
    if (!pointer)
        return;
    Header* header = (Header*)((uintptr_t)pointer - sizeof(Header));
    uint32_t level = IRQ_Disable();
    insert_chunk(header->start, header->size);
    IRQ_Restore(level);
}

void* realloc(void* pointer, size_t size) {
    if (!pointer)
        return malloc(size);
    Header* header = (Header*)((uintptr_t)pointer - sizeof(Header));
    size_t old_size = header->start + header->size - (uintptr_t)pointer;
    if (size <= old_size)
        return pointer;
    void* new_pointer = malloc(size);
    if (new_pointer) {
        memcpy(new_pointer, pointer, old_size);
        free(pointer);
    }
    return new_pointer;
}
