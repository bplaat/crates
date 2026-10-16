// EXIT: 0
// OUT: equals: true
// OUT: hash: 42
// OUT: key equals: true
// OUT: key hash: 42

#include <Object.hh>

class MyKey : IEquatable, IHashable {
    @init i64 value;

    virtual bool equals(Object* other);
    virtual u32 hash();
};
bool MyKey::equals(Object* other) {
    return this->value == ((MyKey*)other)->value;
}
u32 MyKey::hash() {
    return (u32)this->value;
}

int main(void) {
    MyKey* a = my_key_new(42);
    MyKey* b = my_key_new(42);

    printf("equals: %s\n", my_key_equals(a, (Object*)b) ? "true" : "false");
    printf("hash: %u\n", my_key_hash(a));

    // IKeyable auto-implemented: MyKey satisfies IEquatable + IHashable (IKeyable's parents)
    IKeyable k = cast<IKeyable>(a);
    printf("key equals: %s\n", i_keyable_equals(k, (Object*)b) ? "true" : "false");
    printf("key hash: %u\n", i_keyable_hash(k));

    my_key_free(a);
    my_key_free(b);
    return EXIT_SUCCESS;
}
