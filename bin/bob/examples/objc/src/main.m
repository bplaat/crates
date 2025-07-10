#import <Foundation/Foundation.h>
#import <animals/Cat.h>

int main(void) {
    @autoreleasepool {
        NSLog(@"Hello World!");

        Cat *cat = [[Cat alloc] initWithName:@"Whiskers"];
        [cat greet];
    }
    return EXIT_SUCCESS;
}
