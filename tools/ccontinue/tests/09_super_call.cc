// EXIT: 0
// OUT: base value: 10
// OUT: extra: 20
// OUT: base value: 10
// OUT: extra: 20

class Base {
    @init i32 value;
    void greet();
};
void Base::greet() {
    printf("base value: %d\n", this->value);
}

class Child : Base {
    @init i32 extra;
    void greet();
};
void Child::greet() {
    Base::greet();
    printf("extra: %d\n", this->extra);
}

int main(void) {
    Child* c = child_new(10, 20);
    child_greet(c);
    base_greet(c);
    printf("extra: %d\n", c->extra);
    child_free(c);
    return EXIT_SUCCESS;
}
