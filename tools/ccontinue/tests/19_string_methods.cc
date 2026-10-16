// EXIT: 0
// OUT: cstr=hello world
// OUT: length=11
// OUT: contains=true
// OUT: starts_with=true
// OUT: ends_with=true
// OUT: upper=HELLO WORLD
// OUT: lower=hello world
// OUT: trim=hello world
// OUT: index_of=6
// OUT: substring=world

#include <String.hh>

int main(void) {
    String* s = @"hello world";
    printf("cstr=%s\n", string_get_cstr(s));
    printf("length=%zu\n", string_get_length(s));
    printf("contains=%s\n", string_contains(s, "world") ? "true" : "false");
    printf("starts_with=%s\n", string_starts_with(s, "hello") ? "true" : "false");
    printf("ends_with=%s\n", string_ends_with(s, "world") ? "true" : "false");

    String* upper = string_to_upper(s);
    printf("upper=%s\n", string_get_cstr(upper));
    string_free(upper);

    String* lower = string_to_lower(s);
    printf("lower=%s\n", string_get_cstr(lower));
    string_free(lower);

    String* to_trim = @"  hello world  ";
    String* trimmed = string_trim(to_trim);
    printf("trim=%s\n", string_get_cstr(trimmed));
    string_free(trimmed);
    string_free(to_trim);

    printf("index_of=%d\n", string_index_of(s, "world"));

    String* sub = string_substring(s, 6, 5);
    printf("substring=%s\n", string_get_cstr(sub));
    string_free(sub);

    string_free(s);
    return EXIT_SUCCESS;
}
