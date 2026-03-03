#pragma once

#include "Object.hh"

class String : IKeyable {
    @get @init(strdup) @deinit char* cstr;
    @get usize length = strlen(cstr);

    virtual bool equals(Object* other);
    virtual uint32_t hash();
};
