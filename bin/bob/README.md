<img align="right" src="docs/bob-the-builder.jpg" width="200" alt="Bob the Builder">

# Bassie's Obvious Builder (bob)

A simple build system for my projects, because I really like the simplicity of Cargo and I detest all the other build systems :^)

## Supported project types

-   Clang/GCC languages (.c, .cpp, .m, .mm, .s, .asm)
-   JVM languages (.java, .kt)
-   Android (.java, .kt, .xml)

### Supported package types

-   Java Archive (.jar)
-   macOS bundle (.app)
-   Android APK (.apk)

### Supported unit test libraries

-   CUnit (.c, .cpp, .m, .mm)
-   JUnit (.java, .kt)

## Getting Started

### Creating a C project

-   Install bob

    ```sh
    cargo install --git https://github.com/bplaat/crates bob
    ```

-   Create a `bob.toml` manifest file in your project root:

    ```toml
    [project]
    name = "hello"
    version = "0.1.0"
    ```

-   Create a `src/main.c` source file:

    ```c
    #include <stdio.h>
    int main(void) {
        printf("Hello bob!\n");
        return 0;
    }
    ```

-   Then run `bob` to build and run the project:

    ```sh
    bob run
    ```

### Running C unit tests

-   You can create unit tests inline in your C code, for example if you have this function:

    ```c
    int add(int a, int b) {
        return a + b;
    }
    ```

-   You can create a unit test for it like this:

    ```c
    #ifdef TEST
    #include <CUnit/Basic.h>
    void test_add(void) {
        CU_ASSERT_EQUAL(add(3, 4), 7);
        CU_ASSERT_EQUAL(add(-5, 6), 1);
    }
    #endif
    ```

-   Then run `bob` to build and run the tests:

    ```sh
    bob test
    ```

-   This will also print the CUnit test report

## License

Copyright © 2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
