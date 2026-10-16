#import <Foundation/Foundation.h>
#include <mul/mul.h>
#include "Person.hh"

int main(void) {
    @autoreleasepool {
        Person bastiaan("Bastiaan");
        bastiaan.greet();

        NSLog(@"Hello mul %d!\n", mul(3, 4));
    }
    return EXIT_SUCCESS;
}
