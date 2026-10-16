#include <animals/Cat.hpp>
#include <animals/Dog.hpp>
#include <cstdlib>

#include "Person.hpp"

int main(void) {
    Person bastiaan("Bastiaan");
    bastiaan.greet();

    Cat cat("Mittens");
    cat.greet();

    Dog dog("Rover");
    dog.greet();

    return EXIT_SUCCESS;
}
