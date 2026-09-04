# Threadpool Rust library

A dependency-free thread pool for running jobs on a fixed or scalable set of worker threads.

## Getting Started

Create a scalable thread pool and submit jobs with `execute`:

```rust
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

let pool = ThreadPool::new();
let values = Arc::new(Mutex::new(Vec::new()));

for value in 0..4 {
    let values = Arc::clone(&values);
    pool.execute(move || {
        values.lock().expect("Can't lock values").push(value);
    });
}

pool.join();
assert_eq!(values.lock().expect("Can't lock values").len(), 4);
```

The default pool starts with one worker thread per available CPU, grows to eight times that number
when all current worker threads are busy, and closes excess worker threads after they have been idle
for 30 seconds.

## Custom Options

Use `ThreadPool::new_with_options` to control the minimum and maximum number of worker threads and
their worker timeout:

```rust
use std::time::Duration;
use threadpool::{Options, ThreadPool};

let pool = ThreadPool::new_with_options(Options {
    min_worker_threads: 2,
    max_worker_threads: 16,
    worker_timeout: Duration::from_secs(10),
});

pool.execute(|| println!("Hello from the thread pool"));
pool.join();
```

Use `ThreadPool::new_fixed` when the pool must always use a fixed number of worker threads:

```rust
use threadpool::ThreadPool;

let pool = ThreadPool::new_fixed(4);
pool.execute(|| println!("Hello from a fixed thread pool"));
pool.join();
```

## License

Copyright © 2023-2026 Bastiaan van der Plaat

Licensed under the [MIT](../../LICENSE) license.
