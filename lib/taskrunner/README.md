# taskrunner

A minimal, Ninja-inspired parallel task runner for build systems.

Provides a DAG-based task scheduler with:

- Incremental builds via mtime + SHA-1 content hashing
- Parallel execution with condvar-based dependency waiting (no spin-loops)
- Cycle detection via Kahn's topological sort
- Compacting build log (deduplicated on load, rewritten on open)
- Task actions: phony, file copy, shell command, Unix socket message

## Usage

```rust
use taskrunner::{ExecutorBuilder, TaskAction};

let mut builder = ExecutorBuilder::new();
builder.add_task_cmd(
    "cc -c foo.c -o foo.o".to_string(),
    vec!["foo.c".to_string()],
    vec!["foo.o".to_string()],
);
builder.add_task_cmd(
    "cc foo.o -o foo".to_string(),
    vec!["foo.o".to_string()],
    vec!["foo".to_string()],
);

let mut executor = builder.build("target/build.log");
executor.execute(false, None);
```
