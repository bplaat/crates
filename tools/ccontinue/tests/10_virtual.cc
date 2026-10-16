// EXIT: 0
// OUT: Hello, World!
// OUT: *** Hello, Alice! ***
// OUT: *** Hello, Alice! ***

class Greeter {
    @init char* who;
    virtual void greet();
};
void Greeter::greet() {
    printf("Hello, %s!\n", this->who);
}

class FancyGreeter : Greeter {
    virtual void greet();
};
void FancyGreeter::greet() {
    printf("*** Hello, %s! ***\n", this->who);
}

int main(void) {
    Greeter* g = greeter_new("World");
    FancyGreeter* fg = fancy_greeter_new("Alice");
    greeter_greet(g);
    greeter_greet(fg);
    fancy_greeter_greet(fg);
    greeter_free(g);
    fancy_greeter_free(fg);
    return EXIT_SUCCESS;
}
