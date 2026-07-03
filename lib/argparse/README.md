# Argparse crate

A small derive-based command line argument parser for this workspace.

```rs
use argparse::{Parser, Subcommand};

#[derive(Clone, Copy, PartialEq, Eq, Subcommand)]
enum Command {
    #[arg(default, help = "Print this help message")]
    Help,
    #[arg(help = "Print the version number")]
    Version,
    #[arg(help = "Run the program")]
    Run,
}

#[derive(Parser)]
#[arg(name = "demo")]
struct Args {
    #[arg(subcommand)]
    command: Command,
    #[arg(short = 'o', long = "output", value = "file", help = "Write output to file")]
    output: Option<String>,
}
```

## License

Copyright (c) 2026 Bastiaan van der Plaat

Licensed under the [MIT](../../LICENSE) license.
