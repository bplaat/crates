#include "mul.h"

int mul(int a, int b) {
    return a * b;
}

// MARK: Tests
#ifdef TEST

#include <CUnit/Basic.h>

void test_mul(void) {
    CU_ASSERT_EQUAL(mul(2, 3), 6);
    CU_ASSERT_EQUAL(mul(-1, 1), -1);
    CU_ASSERT_EQUAL(mul(0, 5), 0);
    CU_ASSERT_EQUAL(mul(-7, -4), 28);
}

void test_mul_invalid(void) {
    CU_ASSERT_NOT_EQUAL(mul(2, 3), 7);
    CU_ASSERT_NOT_EQUAL(mul(-1, 1), 0);
    CU_ASSERT_NOT_EQUAL(mul(0, 5), 5);
    CU_ASSERT_NOT_EQUAL(mul(-7, -4), -28);
}

#endif
