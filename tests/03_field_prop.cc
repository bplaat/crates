// EXIT: 0
// OUT: value=10
// OUT: value=42

class Counter {
    @prop @init i32 value;
};

int main(void) {
    Counter* c = counter_new(10);
    printf("value=%d\n", counter_get_value(c));
    counter_set_value(c, 42);
    printf("value=%d\n", counter_get_value(c));
    counter_free(c);
    return EXIT_SUCCESS;
}
