#include "Person.hpp"

#include <cstdio>

Person::Person(const char* name) : m_name(name) {}

void Person::greet() {
    printf("Hello %s!\n", m_name);
}

// MARK: Tests
#ifdef TEST

#include <CUnit/Basic.h>

extern "C" void test_person_name(void) {
    Person p("Alice");
    CU_ASSERT_STRING_EQUAL(p.name(), "Alice");
}

#endif
