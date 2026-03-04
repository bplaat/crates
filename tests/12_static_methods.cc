// EXIT: 0
// OUT: add=8
// OUT: multiply=15
// OUT: max=7

class Math {
    static i32 add(i32 a, i32 b);
    static i32 multiply(i32 a, i32 b);
    static i32 max(i32 a, i32 b);
};
i32 Math::add(i32 a, i32 b) {
    return a + b;
}
i32 Math::multiply(i32 a, i32 b) {
    return a * b;
}
i32 Math::max(i32 a, i32 b) {
    return a > b ? a : b;
}

int main(void) {
    printf("add=%d\n", math_add(5, 3));
    printf("multiply=%d\n", math_multiply(3, 5));
    printf("max=%d\n", math_max(3, 7));
    return EXIT_SUCCESS;
}
