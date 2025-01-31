#include "add.h"

int add(int a, int b) {
    return a + b;
}

// MARK: Tests
#ifdef TEST

#include <CUnit/Basic.h>

void test_add(void) {
    CU_ASSERT_EQUAL(add(3, 4), 7);
    CU_ASSERT_EQUAL(add(-1, 1), 0);
    CU_ASSERT_EQUAL(add(0, -0), 0);
    CU_ASSERT_EQUAL(add(-3, -4), -7);
}

void test_add_invalid(void) {
    CU_ASSERT_NOT_EQUAL(add(3, 4), 8);
    CU_ASSERT_NOT_EQUAL(add(-1, 1), 1);
    CU_ASSERT_NOT_EQUAL(add(0, 0), 1);
    CU_ASSERT_NOT_EQUAL(add(-3, -4), -6);
}

#endif
