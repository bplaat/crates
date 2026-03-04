// EXIT: 0
// OUT: Alice
// OUT: Bob
// OUT: Carol
// OUT: size=3

#include <List.hh>
#include <String.hh>

int main(void) {
    List* names = list_new();
    list_add(names, @"Alice");
    list_add(names, @"Bob");
    list_add(names, @"Carol");

    for (String* name in names) {
        printf("%s\n", string_get_cstr(name));
    }
    printf("size=%zu\n", list_get_size(names));

    list_free(names);
    return EXIT_SUCCESS;
}
