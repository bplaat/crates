#include "sub.h"

int sub(int a, int b) {
    return a - b;
}

// MARK: Tests
#ifdef TEST

#include <CUnit/Basic.h>

void test_sub(void) {
    CU_ASSERT_EQUAL(sub(3, 4), -1);
    CU_ASSERT_EQUAL(sub(-1, 1), -2);
    CU_ASSERT_EQUAL(sub(0, 0), 0);
    CU_ASSERT_EQUAL(sub(-3, -4), 1);
}

void test_sub_invalid(void) {
    CU_ASSERT_NOT_EQUAL(sub(3, 4), 0);
    CU_ASSERT_NOT_EQUAL(sub(-1, 1), 0);
    CU_ASSERT_NOT_EQUAL(sub(0, 0), 1);
    CU_ASSERT_NOT_EQUAL(sub(-3, -4), -2);
}

#endif
