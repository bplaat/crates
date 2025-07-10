#include <cstdlib>
#include <animals/Cat.hpp>
#include <animals/Dog.hpp>
#include "Person.hh"

int main(void) {
    @autoreleasepool {
        Person bastiaan("Bastiaan");
        bastiaan.greet();

        Cat cat("Mittens");
        cat.greet();

        Dog dog("Rover");
        dog.greet();
    }
    return EXIT_SUCCESS;
}
