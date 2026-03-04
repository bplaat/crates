/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include "Object.hh"

class String : IKeyable {
    @get @init(strdup) @deinit char* cstr;
    @get usize length = strlen(cstr);

    virtual bool equals(Object* other);
    virtual u32 hash();
    bool contains(char* substr);
    bool starts_with(char* prefix);
    bool ends_with(char* suffix);
    String* to_upper();
    String* to_lower();
    String* trim();
    i32 index_of(char* substr);
    String* substring(usize start, usize length);
};
