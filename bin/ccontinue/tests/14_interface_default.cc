// EXIT: 0
// OUT: Good day!
// OUT: Good day!
// OUT: Good day!

class IGreeter {
    void greet();
    void greet_twice();
};
void IGreeter::greet_twice() {
    greet(this);
    greet(this);
}

class Polite : IGreeter {
    virtual void greet();
};
void Polite::greet() {
    (void)this;
    printf("Good day!\n");
}

int main(void) {
    Polite* p = polite_new();
    IGreeter ig = cast<IGreeter>(p);
    i_greeter_greet(ig);
    i_greeter_greet_twice(ig);
    polite_free(p);
    return EXIT_SUCCESS;
}
