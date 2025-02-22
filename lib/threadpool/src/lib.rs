/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [threadpool](https://crates.io/crates/threadpool

#![forbid(unsafe_code)]

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

// MARK: ThreadPool
/// Thread pool for executing tasks on multiple worker threads
pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

// Define the implementation of ThreadPool
impl ThreadPool {
    /// Creates a new ThreadPool with the specified number of worker threads.
    /// Panics if num_workers is 0.
    pub fn new(num_workers: usize) -> ThreadPool {
        assert!(num_workers > 0, "Number of workers must be greater than 0");

        let (sender, receiver) = channel::<Box<dyn FnOnce() + Send + 'static>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_workers);

        // Spawn worker threads
        for _ in 0..num_workers {
            let receiver = Arc::clone(&receiver);
            let handle = thread::spawn(move || {
                // Worker loop: receive and execute tasks
                loop {
                    let task = {
                        let receiver = receiver.lock().expect("Mutex lock failed");
                        match receiver.recv() {
                            Ok(task) => task,
                            Err(_) => return, // Channel closed, exit loop
                        }
                    };
                    task();
                }
            });
            workers.push(handle);
        }

        ThreadPool { workers, sender }
    }

    /// Executes a closure on an available worker thread.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Box::new(f))
            .expect("A worker thread has died");
    }

    /// Waits for all worker threads to finish their tasks.
    /// This consumes the ThreadPool since it joins all threads.
    pub fn join(self) {
        // Drop the sender to close the channel
        drop(self.sender);

        // Join each worker thread
        for handle in self.workers {
            handle.join().expect("A worker thread panicked");
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;

    #[test]
    #[should_panic]
    fn test_threadpool_zero_workers() {
        let _pool = ThreadPool::new(0); // Should panic due to assert
    }

    #[test]
    fn test_execute_single_task() {
        let pool = ThreadPool::new(1); // Single worker
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = Arc::clone(&counter);
        pool.execute(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        pool.join();
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Task should increment counter
    }

    #[test]
    fn test_execute_multiple_tasks() {
        let pool = ThreadPool::new(2); // Two workers
        let counter = Arc::new(AtomicUsize::new(0));
        let task_count = 10;

        for _ in 0..task_count {
            let counter_clone = Arc::clone(&counter);
            pool.execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10)); // Simulate some work
            });
        }

        pool.join();
        assert_eq!(counter.load(Ordering::SeqCst), task_count); // All tasks should run
    }

    #[test]
    fn test_parallel_execution() {
        let pool = ThreadPool::new(2); // Two workers
        let start = Arc::new(AtomicUsize::new(0));
        let end = Arc::new(AtomicUsize::new(0));

        let start_clone = Arc::clone(&start);
        let end_clone = Arc::clone(&end);
        pool.execute(move || {
            start_clone.store(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(100));
            end_clone.store(1, Ordering::SeqCst);
        });

        let start_clone = Arc::clone(&start);
        let end_clone = Arc::clone(&end);
        pool.execute(move || {
            while start_clone.load(Ordering::SeqCst) == 0 {
                thread::yield_now(); // Wait for first task to start
            }
            assert_eq!(end_clone.load(Ordering::SeqCst), 0); // First task not finished yet
        });

        pool.join();
        assert_eq!(end.load(Ordering::SeqCst), 1); // First task completed
    }
}
