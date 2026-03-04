#include "String.hh"

bool String::equals(Object* other) {
    if (other == NULL || !instanceof<String>(other))
        return false;
    return strcmp(this->cstr, ((String*)other)->cstr) == 0;
}

u32 String::hash() {
    return fnv1a_32(this->cstr, this->length);
}

bool String::contains(char* substr) {
    return strstr(this->cstr, substr) != NULL;
}

bool String::starts_with(char* prefix) {
    return strncmp(this->cstr, prefix, strlen(prefix)) == 0;
}

bool String::ends_with(char* suffix) {
    usize suffix_len = strlen(suffix);
    if (suffix_len > this->length)
        return false;
    return strcmp(this->cstr + this->length - suffix_len, suffix) == 0;
}

String* String::to_upper() {
    char* result = strdup(this->cstr);
    for (usize i = 0; result[i]; i++)
        result[i] = (char)toupper((unsigned char)result[i]);
    String* s = string_new(result);
    free(result);
    return s;
}

String* String::to_lower() {
    char* result = strdup(this->cstr);
    for (usize i = 0; result[i]; i++)
        result[i] = (char)tolower((unsigned char)result[i]);
    String* s = string_new(result);
    free(result);
    return s;
}

String* String::trim() {
    const char* start = this->cstr;
    while (*start == ' ' || *start == '\t' || *start == '\n' || *start == '\r')
        start++;
    const char* end = this->cstr + this->length;
    while (end > start && (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\n' || end[-1] == '\r'))
        end--;
    usize new_len = (usize)(end - start);
    char* result = malloc(new_len + 1);
    memcpy(result, start, new_len);
    result[new_len] = '\0';
    String* s = string_new(result);
    free(result);
    return s;
}

i32 String::index_of(char* substr) {
    char* found = strstr(this->cstr, substr);
    if (found == NULL)
        return -1;
    return (i32)(found - this->cstr);
}

String* String::substring(usize start, usize length) {
    if (start >= this->length)
        return string_new("");
    usize actual = MIN(length, this->length - start);
    char* result = malloc(actual + 1);
    memcpy(result, this->cstr + start, actual);
    result[actual] = '\0';
    String* s = string_new(result);
    free(result);
    return s;
}
