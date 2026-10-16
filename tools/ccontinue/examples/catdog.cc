/*
 * Copyright (c) 2021-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include <List.hh>
#include <String.hh>

// Animal
class Animal {
    @get @init(strdup) @deinit char* name;
    virtual void jump() = 0;
};

// Cat
class Cat : Animal {
    @init i32 lives;
    virtual void jump();
};
void Cat::jump() {
    printf("Cat %s jumps, it has %d lives left!\n", this->name, this->lives);
}

// Dog
class Dog : Animal {
    virtual void jump();
};
void Dog::jump() {
    printf("Dog %s jumps!\n", this->name);
}

// Main
int main(void) {
    List* animals = list_new();
    list_add(animals, cat_new("Mew", 6));
    list_add(animals, dog_new("Woof"));
    list_add(animals, cat_new("Mew 2.0", 9));
    list_add(animals, dog_new("Doggie"));

    for (Animal* animal in animals) {
        animal_jump(animal);
    }

    // instanceof checks
    if (instanceof<Cat>(list_get(animals, 0))) {
        printf("First animal is a cat\n");
    } else {
        printf("First animal is not a cat\n");
    }
    if (instanceof<Dog>(list_get(animals, 0))) {
        printf("First animal is a dog\n");
    } else {
        printf("First animal is not a dog\n");
    }

    list_free(animals);
    return EXIT_SUCCESS;
}
