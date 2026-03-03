// IComparable
class IComparable : IEquatable {
    i32 compare(Object* other);
    bool less_than(Object* other);
    bool greater_than(Object* other);
};
bool IComparable::less_than(Object* other) {
    return compare(this, other) < 0;
}
bool IComparable::greater_than(Object* other) {
    return compare(this, other) > 0;
}

// Number
class Number : IComparable {
    @get @init i32 value;

    virtual bool equals(Object* other);
    virtual i32 compare(Object* other);
};
bool Number::equals(Object* other) {
    if (other == NULL)
        return false;
    return this->value == ((Number*)other)->value;
}
i32 Number::compare(Object* other) {
    return this->value - ((Number*)other)->value;
}

// Main
int main(void) {
    Number* a = number_new(3);
    Number* b = number_new(7);

    // Direct method calls
    printf("a=%d b=%d\n", number_get_value(a), number_get_value(b));
    printf("a == b: %s\n", number_equals(a, (Object*)b) ? "true" : "false");
    printf("a < b: %s\n", number_compare(a, (Object*)b) < 0 ? "true" : "false");

    // Via IComparable
    IComparable c_a = cast<IComparable>(a);
    IComparable c_b = cast<IComparable>(b);
    printf("c_a < c_b (default less_than): %s\n", i_comparable_less_than(c_a, (Object*)b) ? "true" : "false");
    printf("c_a > c_b (default greater_than): %s\n", i_comparable_greater_than(c_a, (Object*)b) ? "true" : "false");
    printf("c_b > c_a (default greater_than): %s\n", i_comparable_greater_than(c_b, (Object*)a) ? "true" : "false");

    // Via IEquatable
    IEquatable e_a = cast<IEquatable>(a);
    printf("equatable a == b: %s\n", i_equatable_equals(e_a, (Object*)b) ? "true" : "false");

    // instanceof checks
    if (instanceof<IComparable>(a)) {
        printf("a is comparable\n");
    } else {
        printf("a is not comparable\n");
    }
    if (instanceof<IEquatable>(a)) {
        printf("a is equatable\n");
    } else {
        printf("a is not equatable\n");
    }
    if (instanceof<IKeyable>(a)) {
        printf("a is keyable\n");
    } else {
        printf("a is not keyable\n");
    }

    number_free(a);
    number_free(b);
    return EXIT_SUCCESS;
}
