#import <Foundation/Foundation.h>

@interface Cat : NSObject {
    NSString* _name;
}

- (instancetype)initWithName:(NSString*)name;
- (void)greet;

@end
