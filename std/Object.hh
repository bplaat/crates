#pragma once

#include "prelude.h"

// Object
class Object {
    usize refs = 1;

    void init();
    virtual void deinit();
    Self* ref();
    void free();
};

// Interfaces
class IEquatable {
    bool equals(Object* other);
};

class IHashable {
    uint32_t hash();
};

class IKeyable : IEquatable, IHashable {};
