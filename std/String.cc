#include <String.hh>

bool String::equals(Object* other) {
    if (other == NULL)
        return false;
    if (!instanceof<String>(other))
        return false;
    return strcmp(this->cstr, ((String*)other)->cstr) == 0;
}

uint32_t String::hash() {
    u32 hash = 2166136261u;
    u8* ptr = (u8*)this->cstr;
    while (*ptr) {
        hash ^= *ptr++;
        hash *= 16777619u;
    }
    return hash;
}
