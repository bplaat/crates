#include <cstdlib>
#include <cstdio>
#include <mul/mul.h>
#include "Person.hpp"

int main(void) {
    Person bastiaan("Bastiaan");
    bastiaan.greet();

    printf("Hello mul %d!\n", mul(3, 4));
    return EXIT_SUCCESS;
}
