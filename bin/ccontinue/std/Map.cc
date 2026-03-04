/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include <Map.hh>

void Map::init() {
    Object::init();
    this->keys = calloc(this->capacity, sizeof(IKeyable));
    this->values = calloc(this->capacity, sizeof(Object*));
}

void Map::deinit() {
    for (usize i = 0; i < this->capacity; i++) {
        if (this->keys[i].obj != NULL) {
            object_free((Object*)this->keys[i].obj);
            if (this->values[i] != NULL)
                object_free(this->values[i]);
        }
    }
    free(this->keys);
    free(this->values);
    Object::deinit();
}

Object* Map::get(IKeyable key) {
    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj))
            return this->values[index];
        index = (index + 1) & (this->capacity - 1);
    }
    return NULL;
}

void Map::set(IKeyable key, Object* value) {
    if (this->filled >= this->capacity * 3 / 4) {
        usize old_capacity = this->capacity;
        this->capacity <<= 1;
        IKeyable* new_keys = calloc(this->capacity, sizeof(IKeyable));
        Object** new_values = calloc(this->capacity, sizeof(Object*));
        for (usize i = 0; i < old_capacity; i++) {
            if (this->keys[i].obj) {
                u32 hash = i_keyable_hash(this->keys[i]);
                usize index = hash & (this->capacity - 1);
                while (new_keys[index].obj)
                    index = (index + 1) & (this->capacity - 1);
                new_keys[index] = this->keys[i];
                new_values[index] = this->values[i];
            }
        }
        free(this->keys);
        free(this->values);
        this->keys = new_keys;
        this->values = new_values;
    }

    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj)) {
            object_free(this->values[index]);
            this->values[index] = value;
            return;
        }
        index = (index + 1) & (this->capacity - 1);
    }
    object_ref((Object*)key.obj);
    this->keys[index] = key;
    this->values[index] = value;
    this->filled++;
}

void Map::remove(IKeyable key) {
    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj)) {
            object_free((Object*)this->keys[index].obj);
            object_free(this->values[index]);
            this->keys[index].obj = NULL;
            this->values[index] = NULL;
            this->filled--;

            // Re-insert displaced entries to maintain linear probe invariant
            usize next = (index + 1) & (this->capacity - 1);
            while (this->keys[next].obj) {
                IKeyable moved_k = this->keys[next];
                Object* moved_v = this->values[next];
                this->keys[next].obj = NULL;
                this->values[next] = NULL;
                this->filled--;
                u32 moved_h = i_keyable_hash(moved_k);
                usize new_idx = moved_h & (this->capacity - 1);
                while (this->keys[new_idx].obj)
                    new_idx = (new_idx + 1) & (this->capacity - 1);
                this->keys[new_idx] = moved_k;
                this->values[new_idx] = moved_v;
                this->filled++;
                next = (next + 1) & (this->capacity - 1);
            }
            return;
        }
        index = (index + 1) & (this->capacity - 1);
    }
}
