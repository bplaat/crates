#include <Set.hh>

void Set::init() {
    Object::init();
    this->keys = calloc(this->capacity, sizeof(IKeyable));
}

void Set::deinit() {
    for (usize i = 0; i < this->capacity; i++) {
        if (this->keys[i].obj != NULL)
            object_free((Object*)this->keys[i].obj);
    }
    free(this->keys);
    Object::deinit();
}

bool Set::contains(IKeyable key) {
    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj))
            return true;
        index = (index + 1) & (this->capacity - 1);
    }
    return false;
}

void Set::add(IKeyable key) {
    if (this->size >= this->capacity * 3 / 4) {
        usize old_capacity = this->capacity;
        this->capacity <<= 1;
        IKeyable* new_keys = calloc(this->capacity, sizeof(IKeyable));
        for (usize i = 0; i < old_capacity; i++) {
            if (this->keys[i].obj) {
                u32 hash = i_keyable_hash(this->keys[i]);
                usize index = hash & (this->capacity - 1);
                while (new_keys[index].obj)
                    index = (index + 1) & (this->capacity - 1);
                new_keys[index] = this->keys[i];
            }
        }
        free(this->keys);
        this->keys = new_keys;
    }

    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj))
            return;
        index = (index + 1) & (this->capacity - 1);
    }
    object_ref((Object*)key.obj);
    this->keys[index] = key;
    this->size++;
}

void Set::remove(IKeyable key) {
    u32 hash = i_keyable_hash(key);
    usize index = hash & (this->capacity - 1);
    while (this->keys[index].obj) {
        if (i_keyable_equals(this->keys[index], (Object*)key.obj)) {
            object_free((Object*)this->keys[index].obj);
            this->keys[index].obj = NULL;
            this->size--;
            return;
        }
        index = (index + 1) & (this->capacity - 1);
    }
}
