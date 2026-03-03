#pragma once

#include "Object.hh"

class Set {
    IKeyable* keys;
    @get usize capacity = 8;
    @get usize size = 0;

    void init();
    virtual void deinit();
    bool contains(IKeyable key);
    void add(IKeyable key);
    void remove(IKeyable key);
};
