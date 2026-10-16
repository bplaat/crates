// EXIT: 0
// OUT: str=hello
// OUT: int=42
// OUT: float=3.14
// OUT: bool=true

#include <String.hh>

int main(void) {
    String* s = @"hello";
    printf("str=%s\n", string_get_cstr(s));
    string_free(s);

    Int* i = @42;
    printf("int=%lld\n", (long long)int_get_value(i));
    int_free(i);

    Float* f = @3.14;
    printf("float=%.2f\n", float_get_value(f));
    float_free(f);

    Bool* b = @true;
    printf("bool=%s\n", bool_get_value(b) ? "true" : "false");
    bool_free(b);

    return EXIT_SUCCESS;
}
