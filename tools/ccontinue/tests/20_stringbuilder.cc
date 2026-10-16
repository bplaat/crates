// EXIT: 0
// OUT: result=Hello, World!
// OUT: length=13

#include <StringBuilder.hh>

int main(void) {
    StringBuilder* sb = string_builder_new();
    string_builder_append_cstr(sb, "Hello");
    string_builder_append_char(sb, ',');
    string_builder_append_char(sb, ' ');
    String* w = @"World!";
    string_builder_append_string(sb, w);
    string_free(w);

    String* result = string_builder_build(sb);
    printf("result=%s\n", string_get_cstr(result));
    printf("length=%zu\n", string_get_length(result));
    string_free(result);

    string_builder_free(sb);
    return EXIT_SUCCESS;
}
