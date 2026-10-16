/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include "Object.hh"

class Map {
    IKeyable* keys;
    Object** values;
    @get usize capacity = 8;
    @get usize filled = 0;

    void init();
    virtual void deinit();
    Object* get(IKeyable key);
    void set(IKeyable key, Object* value);
    void remove(IKeyable key);
};
