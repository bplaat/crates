// EXIT: 0
// OUT: get alice=Alice
// OUT: get bob=Bob
// OUT: after remove alice=not found
// OUT: filled=1

#include <Map.hh>
#include <String.hh>

int main(void) {
    Map* map = map_new();
    String* k_alice = @"alice";
    String* k_bob = @"bob";

    map_set(map, cast<IKeyable>(k_alice), @"Alice");
    map_set(map, cast<IKeyable>(k_bob), @"Bob");

    String* v_alice = (String*)map_get(map, cast<IKeyable>(k_alice));
    printf("get alice=%s\n", string_get_cstr(v_alice));

    String* v_bob = (String*)map_get(map, cast<IKeyable>(k_bob));
    printf("get bob=%s\n", string_get_cstr(v_bob));

    map_remove(map, cast<IKeyable>(k_alice));
    String* v_gone = (String*)map_get(map, cast<IKeyable>(k_alice));
    printf("after remove alice=%s\n", v_gone ? string_get_cstr(v_gone) : "not found");

    printf("filled=%zu\n", map_get_filled(map));

    map_free(map);
    string_free(k_alice);
    string_free(k_bob);
    return EXIT_SUCCESS;
}
