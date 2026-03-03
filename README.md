# cContinue Transpiler

_A transpiler that translates an OOP-extension for the C programming language back to C_

I like C and I like C++, both are powerful languages in their own right. But C++ is quite complicated and sometimes I just want to create some classes with inheritance in my C project. So I've created this weird hacky Python transpiler that translates a C++ like syntax with a Java like class system back to C code 🤓.

## Usage

```sh
./ccc.py <file.cc> [options]
```

| Flag        | Description                                        |
| ----------- | -------------------------------------------------- |
| `-o <file>` | Output file                                        |
| `-I <path>` | Add include search path                            |
| `-S`        | Only run the transpile step (emit `.c` source)     |
| `-c`        | Only transpile and compile (emit `.o` object)      |
| `-r`        | Run the linked binary after building               |
| `-R`        | Run with memory leak checks (`leaks` / `valgrind`) |

## Syntax

### Classes

Classes implicitly extend `Object` (heap-allocated, reference-counted). Methods are declared in the class body and implemented with `ClassName::method()`. Use `this` inside methods, call parent methods with `ParentClass::method()`, and use `Self*` as return type when returning the current instance:

```cpp
class Person {
    @get @init(strdup) @deinit char* name;
    @prop @init i32 age;
    void greet();
};

void Person::greet() {
    printf("Hello %s, you are %d years old!\n", this->name, this->age);
}

int main(void) {
    Person* person = person_new("Bastiaan", 21);
    person_greet(person);
    person_free(person);
}
```

### Field attributes

Attributes before a field declaration auto-generate boilerplate:

| Attribute     | Effect                                                    |
| ------------- | --------------------------------------------------------- |
| `@get`        | Generate a `get_<field>()` getter                         |
| `@set`        | Generate a `set_<field>()` setter                         |
| `@prop`       | Alias for `@get` + `@set`                                 |
| `@init`       | Pass field as argument to `_new()`; assign directly       |
| `@init(fn)`   | Same but wrap the argument with `fn(arg)` (e.g. `strdup`) |
| `@deinit`     | Free field in `deinit`; calls `free()` by default         |
| `@deinit(fn)` | Same but call `fn(field)` instead of `free()`             |

### Inheritance & virtual methods

A class can extend **one** parent with `: Parent`. Mark methods `virtual` for vtable dispatch. A class with `virtual method = 0` is abstract:

```cpp
class Animal {
    @get @init(strdup) @deinit char* name;
    virtual void speak() = 0;
};

class Dog : Animal {
    virtual void speak();
};
void Dog::speak() {
    printf("Dog %s barks!\n", this->name);
}
```

### Interfaces

Interfaces are declared as `class IFoo` (name starts with `I` + uppercase). Implement with `: IFace`. Methods must **not** have bodies inside the declaration — define defaults outside with `IFoo::method()`. Dispatch via `cast<IFoo>(obj)` (fat pointer). Interfaces can extend other interfaces (multi-parent):

```cpp
class IEquatable {
    virtual bool equals(Object* other) = 0;
};

class IComparable : IEquatable {
    i32 compare(Object* other);
    bool less_than(Object* other);
    bool greater_than(Object* other);
};
bool IComparable::less_than(Object* other) { return compare(this, other) < 0; }
bool IComparable::greater_than(Object* other) { return compare(this, other) > 0; }

class Number : IComparable {
    @get @init i32 value;

    virtual bool equals(Object* other);
    virtual i32 compare(Object* other);
};

Number* n = number_new(42);
IComparable c = cast<IComparable>(n);
IEquatable  e = cast<IEquatable>(n);
```

Generated dispatch macros use the snake_case of the interface name:

- `IComparable` → `i_comparable_less_than(c, other)`
- `IHashable` → `i_hashable_hash(h)`

### Literals

`@"..."`, `@true`/`@false`, `@<int>`, and `@<float>` are sugar for the corresponding boxed-type constructors:

```cpp
String* s = @"hello";  // → string_new("hello")
Bool*   b = @true;     // → bool_new(true)
Int*    i = @42;       // → int_new(42)
Float*  f = @3.14;     // → float_new(3.14)
```

### Type checks

`instanceof<Type>(expr)` returns a `bool`. Checks exact class for classes, interface slot for interfaces:

```cpp
if (instanceof<IHashable>(obj)) { ... }
if (instanceof<Dog>(obj)) { ... }
```

## Standard library

Include stdlib classes with `#include <ClassName.hh>`.

### `Object` — base class

Every class implicitly extends `Object`. Provides reference counting:

```c
Object* object_ref(Object* obj)   // Increment reference count; returns obj
void    object_free(Object* obj)  // Decrement reference count; frees when it reaches zero
```

### `Bool` — heap boolean

```c
Bool* bool_new(bool value)        // Create a new Bool
void  bool_free(Bool* b)          // Free the Bool
bool  bool_get_value(Bool* b)     // Get the raw bool value
```

### `Int` — heap integer

```c
Int* int_new(i32 value)          // Create a new Int
void int_free(Int* i)            // Free the Int
i32  int_get_value(Int* i)       // Get the raw i32 value
```

### `Float` — heap float

```c
Float* float_new(f32 value)      // Create a new Float
void  float_free(Float* f)       // Free the Float
f32   float_get_value(Float* f)  // Get the raw f32 value
```

### `String` — heap string

Implements `IEquatable`, `IHashable`, and `IKeyable`. Stores an owned copy of the string.

```c
String* string_new(char* cstr)                  // Create a new String (copies cstr with strdup)
void    string_free(String* s)                  // Free the string
char*   string_get_cstr(String* s)              // Get the raw char* pointer
usize   string_get_length(String* s)            // Get the string length
bool    string_equals(String* s, Object* other) // Compare two strings by content
u32     string_hash(String* s)                  // FNV-1a hash of the string
```

### `List` — dynamic array

```cpp
#include <List.hh>
```

A growable array of `Object*` values.

```c
List*    list_new()                                         // Create an empty list
void     list_free(List* list)                              // Free the list and all contained objects
usize    list_get_size(List* list)                          // Number of items
Object*  list_get(List* list, usize index)                  // Get item at index
void     list_set(List* list, usize index, Object* item)    // Set item at index
void     list_add(List* list, Object* item)                 // Append item
void     list_insert(List* list, usize index, Object* item) // Insert item at index
void     list_remove(List* list, usize index)               // Remove item at index
```

### `Map` — hash map

```cpp
#include <Map.hh>
```

A hash map from `IKeyable` keys to `Object*` values. Use `String` (or any `IKeyable` type) as the key:

```cpp
String* k = @"hello";
map_set(map, cast<IKeyable>(k), value);
Object* v = map_get(map, cast<IKeyable>(k));
map_remove(map, cast<IKeyable>(k));
string_free(k);
```

```c
Map*     map_new()                                      // Create an empty map
void     map_free(Map* map)                             // Free the map and all stored values
usize    map_get_capacity(Map* map)                     // Current bucket capacity
usize    map_get_filled(Map* map)                       // Number of stored entries
Object*  map_get(Map* map, IKeyable key)                // Lookup by key; returns NULL if absent
void     map_set(Map* map, IKeyable key, Object* value) // Insert or update entry
void     map_remove(Map* map, IKeyable key)             // Remove entry (frees key and value)
```

## Built-in types

`prelude.h` defines short aliases for the standard integer and float types:

```c
i8, i16, i32, i64  // Signed integers
u8, u16, u32, u64  // Unsigned integers
f32, f64           // Floating point
isize              // ptrdiff_t
usize              // size_t
```

## License

Copyright © 2021-2026 Bastiaan van der Plaat

Licensed under the [MIT](LICENSE) license.
