#import <Cocoa/Cocoa.h>

int main(void) {
    @autoreleasepool {
        NSAlert *alert = [NSAlert new];
        [alert setMessageText:@"Hello macOS!"];
        [alert runModal];
    }
    return EXIT_SUCCESS;
}
