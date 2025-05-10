<img align="right" src="docs/bob-the-builder.jpg" width="200" alt="Bob the Builder">

# Bassie's Obvious Builder (bob)

A simple meta-build system for my projects, because I like the simplicity of Cargo. But meh it's just a ninja build file generator.

## Supported project types

-   Clang/GCC languages (.c, .cpp, .m, .mm)
-   Java (.java)
-   Android (.java, .xml)

### Supported package types

-   Java Jar (.jar)
-   macOS bundle (.app)
-   Android APK (.apk)

## Getting Started

-   Install bob

    ```sh
    cargo install --git https://github.com/bplaat/crates bob
    ```

-   Create a `bob.toml` file in your project root:

    ```toml
    [project]
    name = "hello"
    version = "0.1.0"
    ```

-   Create source files in the `src` directory, for example `main.c`:

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

## Creating simple unit tests (in C/C++)

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
