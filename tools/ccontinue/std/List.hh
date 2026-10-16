/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include "Object.hh"

class List : IIterable {
    Object** items;
    @get usize capacity = 8;
    @get usize size = 0;

    void init();
    virtual void deinit();
    Object* get(usize index);
    void set(usize index, Object* item);
    void add(Object* item);
    void insert(usize index, Object* item);
    void remove(usize index);
    virtual IIterator iterator();
};

class ListIterator : IIterator {
    @init List* list;
    usize index = 0;

    virtual bool has_next();
    virtual Object* next();
};
