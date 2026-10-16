// EXIT: 0
// OUT: dog hash: 00000000
// OUT: cat hash: 00000045
// OUT: interface dog: 00000000
// OUT: interface cat: 00000045

class Dog : IHashable {
    virtual u32 hash();
};
u32 Dog::hash() {
    (void)this;
    return 0;
}

class Cat : IHashable {
    virtual u32 hash();
};
u32 Cat::hash() {
    (void)this;
    return 69;
}

int main(void) {
    Dog* dog = dog_new();
    printf("dog hash: %08x\n", dog_hash(dog));

    Cat* cat = cat_new();
    printf("cat hash: %08x\n", cat_hash(cat));

    IHashable h_dog = cast<IHashable>(dog);
    printf("interface dog: %08x\n", i_hashable_hash(h_dog));

    Object* obj = (Object*)cat;
    IHashable h_cat = cast<IHashable>(obj);
    printf("interface cat: %08x\n", i_hashable_hash(h_cat));

    dog_free(dog);
    cat_free(cat);
    return EXIT_SUCCESS;
}
