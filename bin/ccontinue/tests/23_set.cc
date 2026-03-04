// EXIT: 0
// OUT: contains apple=true
// OUT: contains banana=true
// OUT: after remove apple=false
// OUT: size=2

#include <Set.hh>
#include <String.hh>

int main(void) {
    Set* set = set_new();
    String* apple = @"apple";
    String* banana = @"banana";
    String* cherry = @"cherry";

    set_add(set, cast<IKeyable>(apple));
    set_add(set, cast<IKeyable>(banana));

    printf("contains apple=%s\n", set_contains(set, cast<IKeyable>(apple)) ? "true" : "false");
    printf("contains banana=%s\n", set_contains(set, cast<IKeyable>(banana)) ? "true" : "false");

    set_remove(set, cast<IKeyable>(apple));
    printf("after remove apple=%s\n", set_contains(set, cast<IKeyable>(apple)) ? "true" : "false");

    set_add(set, cast<IKeyable>(cherry));
    printf("size=%zu\n", set_get_size(set));

    set_free(set);
    string_free(apple);
    string_free(banana);
    string_free(cherry);
    return EXIT_SUCCESS;
}
