/*
 * Copyright (c) 2021-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

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
    u32 hash();
};

class IKeyable : IEquatable, IHashable {};

class IIterator {
    bool has_next();
    Object* next();
};

class IIterable {
    IIterator iterator();
};

// Boxed types
class Bool : IEquatable, IHashable {
    @get @init bool value;

    virtual bool equals(Object* other);
    virtual u32 hash();
};

class Int : IEquatable, IHashable {
    @get @init i64 value;

    virtual bool equals(Object* other);
    virtual u32 hash();
};

class Float : IEquatable, IHashable {
    @get @init f64 value;

    virtual bool equals(Object* other);
    virtual u32 hash();
};
