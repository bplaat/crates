#include <List.hh>
#include <Map.hh>
#include <String.hh>

class Person {
    @get @init @deinit String* name;
    @prop @init i32 age;

    void greet();
};

void Person::greet() {
    printf("Hello %s, you are %d years old!\n", string_get_cstr(this->name), this->age);
}

int main(void) {
    // Simple class instance
    Person* bastiaan = person_new(@"Bastiaan", 21);
    person_set_age(bastiaan, person_get_age(bastiaan) + 1);
    person_greet(bastiaan);
    person_free(bastiaan);

    // Build in dynamic lists
    List* persons = list_new();
    list_add(persons, person_new(@"Bastiaan", 21));
    list_add(persons, person_new(@"Sander", 20));
    list_add(persons, person_new(@"Leonard", 17));
    list_add(persons, person_new(@"Jiska", 16));

    for (usize i = 0; i < list_get_size(persons); i++) {
        Person* person = (Person*)list_get(persons, i);
        person_greet(person);
    }

    List* persons_copy = list_ref(persons);
    list_free(persons_copy);

    list_free(persons);

    // Build in dynamic maps
    Map* map = map_new();
    String* k_leonard = @"leonard";
    String* k_sander = @"sander";
    map_set(map, cast<IKeyable>(k_leonard), person_new(@"Leonard", 17));
    map_set(map, cast<IKeyable>(k_sander), person_new(@"Sander", 19));
    map_set(map, cast<IKeyable>(k_sander), person_new(@"Sander", 20));

    Person* leonard = (Person*)map_get(map, cast<IKeyable>(k_leonard));
    person_greet(leonard);

    map_remove(map, cast<IKeyable>(k_sander));
    Person* sander = (Person*)map_get(map, cast<IKeyable>(k_sander));
    printf("Sander is %s\n", sander ? "found" : "not found");

    map_free(map);
    string_free(k_leonard);
    string_free(k_sander);

    return EXIT_SUCCESS;
}
