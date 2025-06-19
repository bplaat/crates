#include <stdlib.h>
#include <stdio.h>
#include <mul/mul.h>
#include "sub/sub.h"
#include "add.h"

int main(void) {
    printf("Hello add %d!\n", add(1, 2));
    printf("Hello sub %d!\n", sub(5, 2));
    printf("Hello mul %d!\n", mul(3, 4));
    return EXIT_SUCCESS;
}
