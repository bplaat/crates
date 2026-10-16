#include "Dog.hpp"

#include <cstdio>

Dog::Dog(const char* name) : m_name(name) {}

void Dog::greet() {
    printf("Woof said %s!\n", m_name);
}
