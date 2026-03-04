#pragma once

#include "Object.hh"
#include "String.hh"

class StringBuilder {
    char* buf;
    @get usize length = 0;
    usize capacity = 16;

    void init();
    virtual void deinit();
    void append_cstr(char* s);
    void append_char(char c);
    void append_string(String* s);
    String* build();
};
