#include "Person.hpp"

#include <cstdio>

Person::Person(const char* name) : m_name(name) {}

void Person::greet() {
    printf("Hello %s!\n", m_name);
}
