// EXIT: 0
// OUT: size=3
// OUT: [0]=first
// OUT: [1]=second
// OUT: [2]=third
// OUT: after remove size=2
// OUT: [0]=first
// OUT: [1]=third

#include <List.hh>
#include <String.hh>

int main(void) {
    List* list = list_new();
    list_add(list, @"first");
    list_add(list, @"second");
    list_add(list, @"third");

    printf("size=%zu\n", list_get_size(list));
    for (usize i = 0; i < list_get_size(list); i++) {
        printf("[%zu]=%s\n", i, string_get_cstr((String*)list_get(list, i)));
    }

    list_remove(list, 1);
    printf("after remove size=%zu\n", list_get_size(list));
    for (usize i = 0; i < list_get_size(list); i++) {
        printf("[%zu]=%s\n", i, string_get_cstr((String*)list_get(list, i)));
    }

    list_free(list);
    return EXIT_SUCCESS;
}
