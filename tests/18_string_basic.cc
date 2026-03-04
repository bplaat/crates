// EXIT: 0
// OUT: cstr=hello
// OUT: length=5
// OUT: equals=true
// OUT: equals=false

#include <String.hh>

int main(void) {
    String* s = string_new("hello");
    printf("cstr=%s\n", string_get_cstr(s));
    printf("length=%zu\n", string_get_length(s));

    String* s2 = string_new("hello");
    printf("equals=%s\n", string_equals(s, (Object*)s2) ? "true" : "false");
    string_free(s2);

    String* s3 = string_new("world");
    printf("equals=%s\n", string_equals(s, (Object*)s3) ? "true" : "false");
    string_free(s3);

    string_free(s);
    return EXIT_SUCCESS;
}
