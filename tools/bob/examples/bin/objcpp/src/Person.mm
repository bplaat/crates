#include "Person.hh"
#import <Foundation/Foundation.h>

Person::Person(const char* name) : m_name(name) {}

void Person::greet() {
    NSLog(@"Hello %s!\n", m_name);
}
