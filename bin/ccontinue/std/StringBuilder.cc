#include <StringBuilder.hh>

void StringBuilder::init() {
    Object::init();
    this->buf = malloc(this->capacity);
    this->buf[0] = '\0';
}

void StringBuilder::deinit() {
    free(this->buf);
    Object::deinit();
}

void StringBuilder::append_cstr(char* s) {
    usize add = strlen(s);
    while (this->length + add + 1 > this->capacity) {
        this->capacity <<= 1;
        this->buf = realloc(this->buf, this->capacity);
    }
    memcpy(this->buf + this->length, s, add + 1);
    this->length += add;
}

void StringBuilder::append_char(char c) {
    if (this->length + 2 > this->capacity) {
        this->capacity <<= 1;
        this->buf = realloc(this->buf, this->capacity);
    }
    this->buf[this->length++] = c;
    this->buf[this->length] = '\0';
}

void StringBuilder::append_string(String* s) {
    string_builder_append_cstr(this, string_get_cstr(s));
}

String* StringBuilder::build() {
    return string_new(this->buf);
}
