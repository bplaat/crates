#import "Cat.h"

@implementation Cat

- (instancetype)initWithName:(NSString*)name {
    self = [super init];
    if (self) {
        _name = [name copy];
    }
    return self;
}

- (void)greet {
    NSLog(@"Miauw said %@!", _name);
}

@end
