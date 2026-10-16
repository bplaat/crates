// EXIT: 0
// OUT: value=42
// OUT: result=47

class Counter {
    @init i32 value;
    i32 add(i32 n);
};

i32 Counter::add(i32 n) {
    return this->value + n;
}

int main(void) {
    Counter* c = counter_new(42);
    printf("value=%d\n", c->value);
    printf("result=%d\n", counter_add(c, 5));
    counter_free(c);
    return EXIT_SUCCESS;
}
