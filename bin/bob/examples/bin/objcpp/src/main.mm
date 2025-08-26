#import <Foundation/Foundation.h>
#import <animals/Cat.h>
#include "Person.hh"

int main(void) {
    @autoreleasepool {
        Person bastiaan("Bastiaan");
        bastiaan.greet();

        Cat* cat = [[Cat alloc] initWithName:@"Whiskers"];
        [cat greet];
    }
    return EXIT_SUCCESS;
}
