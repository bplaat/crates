#include "Person.hh"
#include <cstdlib>

int main(void) {
    @autoreleasepool {
        Person bastiaan("Bastiaan");
        bastiaan.greet();
    }
    return EXIT_SUCCESS;
}
