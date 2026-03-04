// EXIT: 0
// OUT: dog instanceof Dog: true
// OUT: dog instanceof Cat: false
// OUT: dog instanceof IHashable: true
// OUT: cat instanceof IHashable: true
// OUT: dog instanceof IEquatable: false

class Dog : IHashable {
    virtual u32 hash();
};
u32 Dog::hash() {
    (void)this;
    return 1;
}

class Cat : IHashable {
    virtual u32 hash();
};
u32 Cat::hash() {
    (void)this;
    return 2;
}

int main(void) {
    Dog* dog = dog_new();
    Cat* cat = cat_new();
    Object* obj = (Object*)dog;

    printf("dog instanceof Dog: %s\n", instanceof<Dog>(obj) ? "true" : "false");
    printf("dog instanceof Cat: %s\n", instanceof<Cat>(obj) ? "true" : "false");
    printf("dog instanceof IHashable: %s\n", instanceof<IHashable>(obj) ? "true" : "false");
    printf("cat instanceof IHashable: %s\n", instanceof<IHashable>(cat) ? "true" : "false");
    printf("dog instanceof IEquatable: %s\n", instanceof<IEquatable>(dog) ? "true" : "false");

    dog_free(dog);
    cat_free(cat);
    return EXIT_SUCCESS;
}
