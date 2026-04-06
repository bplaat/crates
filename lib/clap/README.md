# clap

Minimal, zero-dependency clap-compatible argument parsing library for the bplaat/crates monorepo.
Provides `#[derive(Parser)]` and `#[derive(Subcommand)]` derive macros that generate inline
parsing code at compile time — no runtime registry or reflection.

## Usage

```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "My tool", version)]
struct Args {
    #[arg(short, long, help = "Verbose output")]
    verbose: bool,

    #[arg(short = 'o', long, help = "Output file", value_name = "FILE")]
    output: Option<String>,

    #[arg(short = 'j', long, alias = "thread-count", value_name = "N")]
    jobs: Option<usize>,

    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Build,
    #[command(alias = "c")]
    Clean,
    #[command(name = "clean-cache")]
    CleanCache,
    Help,
}

fn main() {
    let args = Args::parse();
    // ...
}
```

## Attributes

### Struct-level `#[command(...)]`
| Attribute | Description |
|-----------|-------------|
| `about = "text"` | Description shown in `--help` |
| `version` | Enable `--version` / `-V` (prints `CARGO_PKG_VERSION`) |

### Field-level `#[arg(...)]`
| Attribute | Description |
|-----------|-------------|
| `short` | Use first letter of field name as short flag |
| `short = 'x'` | Custom short flag character |
| `long` | Use field name (snake\_case → kebab-case) as long flag |
| `long = "name"` | Custom long flag name |
| `alias = "name"` | Additional long flag name |
| `help = "text"` | Help text shown in `--help` |
| `default_value = "val"` | Default value (as string, parsed to field type) |
| `value_name = "NAME"` | Placeholder shown in `--help` |
| `subcommand` | Mark field as the subcommand field |

Fields without `short` or `long` are treated as positional arguments.

### Enum variant-level `#[command(...)]`
| Attribute | Description |
|-----------|-------------|
| `name = "kebab-name"` | Override the default kebab-case name |
| `alias = "x"` | Add an alias for this variant |

## Supported field types
| Type | Behavior |
|------|----------|
| `bool` | Presence flag — `true` when the flag is given |
| `String` | Option value or positional argument |
| `Option<T>` | Optional value; `None` if not provided |
| `Vec<T>` | Repeatable option or positional argument collector |
| `T: FromStr` | Any parseable value type |

## Entry points
| Method | Description |
|--------|-------------|
| `Args::parse()` | Parse from `env::args()` (skips binary name) |
| `Args::cargo_parse()` | Like `parse()` but also skips the first non-flag arg (Cargo subcommand delegation) |
