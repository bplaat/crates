#import <Foundation/Foundation.h>
#include <mul/mul.h>

int main(void) {
    @autoreleasepool {
        NSLog(@"Hello mul %d!\n", mul(3, 4));
    }
    return EXIT_SUCCESS;
}
