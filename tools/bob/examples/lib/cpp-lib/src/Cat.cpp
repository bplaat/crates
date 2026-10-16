#include "Cat.hpp"

#include <cstdio>

Cat::Cat(const char* name) : m_name(name) {}

void Cat::greet() {
    printf("Miauw said %s!\n", m_name);
}
