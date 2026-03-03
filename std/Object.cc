#include "Object.hh"

// Object
void Object::init() {}

void Object::deinit() {
    free(this);
}

Self* Object::ref() {
    this->refs++;
    return this;
}

void Object::free() {
    if (--this->refs <= 0)
        object_deinit(this);
}

// Bool
bool Bool::equals(Object* other) {
    if (other == NULL || !instanceof<Bool>(other))
        return false;
    return this->value == ((Bool*)other)->value;
}

u32 Bool::hash() {
    return this->value ? 1 : 0;
}

// Int
bool Int::equals(Object* other) {
    if (other == NULL || !instanceof<Int>(other))
        return false;
    return this->value == ((Int*)other)->value;
}

u32 Int::hash() {
    return fnv1a_32(&this->value, sizeof(this->value));
}

// Float
bool Float::equals(Object* other) {
    if (other == NULL || !instanceof<Float>(other))
        return false;
    return this->value == ((Float*)other)->value;
}

u32 Float::hash() {
    return fnv1a_32(&this->value, sizeof(this->value));
}

// String
bool String::equals(Object* other) {
    if (other == NULL || !instanceof<String>(other))
        return false;
    return strcmp(this->cstr, ((String*)other)->cstr) == 0;
}

u32 String::hash() {
    return fnv1a_32(this->cstr, this->length);
}
