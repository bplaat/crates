<img align="right" src="docs/bob-the-builder.jpg" width="200" alt="Bob the Builder">

# Bassie's Obvious Builder (bob)

A simple meta-build system for my projects, because I like the simplicity of Cargo. But meh it's just a ninja build file generator.

## Supported project types

-   GCC languages (.c, .cpp, .m, .mm)
-   Java (.java)

### Supported package types

-   Java Jar (.jar)
-   macOS bundle (.app)

## Getting Started

-   Install bob

    ```sh
    cargo install --git https://github.com/bplaat/crates bob
    ```

-   Create a `bob.toml` file in your project root

    ```toml
    [project]
    name = "hello"
    version = "0.1.0"
    ```

-   Create source files in `src` directory, for example a `main.c`

    ```c
    #include <stdio.h>
    int main(void) {
        printf("Hello bob!\n");
        return 0;
    }
    ```

-   Then run `bob` to build and run the project

    ```sh
    bob run
    ```
