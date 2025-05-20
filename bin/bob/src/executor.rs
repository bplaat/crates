/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{fs, thread};

use threadpool::ThreadPool;

use crate::log::{Log, LogEntry};
use crate::sha1::sha1;

// MARK: Task
#[derive(Debug)]
pub(crate) struct Task {
    id: usize,
    command: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

// MARK: Executor
#[derive(Debug)]
pub(crate) struct Executor {
    tasks: Vec<Task>,
    task_id_counter: usize,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self {
            tasks: Vec::new(),
            task_id_counter: 0,
        }
    }

    pub(crate) fn add_task(&mut self, command: String, inputs: Vec<String>, outputs: Vec<String>) {
        self.tasks.push(Task {
            id: self.task_id_counter,
            command,
            inputs,
            outputs,
        });
        self.task_id_counter += 1;
    }

    pub(crate) fn execute(&self, log_path: &str) {
        #[cfg(debug_assertions)]
        println!("{:#?}", self.tasks);

        let log = Log::new(log_path);
        let num_threads = thread::available_parallelism().map_or(1, |n| n.get());
        let pool = ThreadPool::new(num_threads);
        self.execute_task(
            self.tasks.last().expect("No tasks provided"),
            &pool,
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(log)),
        );
        pool.join();
    }

    fn execute_task(
        &self,
        task: &Task,
        pool: &ThreadPool,
        scheduled_task_ids: Arc<Mutex<Vec<usize>>>,
        done_task_ids: Arc<Mutex<Vec<usize>>>,
        log: Arc<Mutex<Log>>,
    ) {
        // Check if task is already scheduled
        {
            let mut scheduled_task_ids = scheduled_task_ids.lock().expect("Could not lock mutex");
            if scheduled_task_ids.contains(&task.id) {
                return;
            }
            scheduled_task_ids.push(task.id);
        }

        // Find and run dependencies
        let mut dependency_ids = Vec::new();
        for input in &task.inputs {
            for other_task in &self.tasks {
                if other_task.outputs.contains(input) {
                    dependency_ids.push(other_task.id);
                    self.execute_task(
                        other_task,
                        pool,
                        scheduled_task_ids.clone(),
                        done_task_ids.clone(),
                        log.clone(),
                    );
                }
            }
        }

        let task_id = task.id;
        let task_inputs = task.inputs.clone();
        let task_command = task.command.clone();
        let task_outputs = task.outputs.clone();
        pool.execute(move || {
            // Wait for dependencies to finish
            if !dependency_ids.is_empty() {
                while {
                    let done_task_ids = done_task_ids.lock().expect("Could not lock mutex");
                    !dependency_ids
                        .iter()
                        .all(|dep_id| done_task_ids.contains(dep_id))
                } {
                    thread::yield_now();
                }
            }

            // Check if inputs have changed
            let mut inputs_changed = false;
            for input in &task_inputs {
                let input_path = if fs::metadata(input)
                    .map(|metadata| metadata.is_dir())
                    .unwrap_or(false)
                {
                    format!("{}/.stamp", input)
                } else {
                    input.clone()
                };

                // Get input modified time
                let metadata = fs::metadata(&input_path).unwrap_or_else(|_| {
                    eprintln!("Can't open input file: {}", input);
                    exit(1)
                });
                let modified_time = metadata
                    .modified()
                    .expect("Failed to get modified time")
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                // Calculate input hash
                let buffer = fs::read(&input_path).unwrap_or_else(|_| {
                    eprintln!("Can't read input file: {}", input);
                    exit(1)
                });
                let hash = if !buffer.is_empty() {
                    Some(sha1(&buffer))
                } else {
                    None
                };

                // Check if the input has changed
                {
                    let mut log = log.lock().expect("Could not lock mutex");
                    if log.get(input).is_none_or(|entry| {
                        entry.modified_time != modified_time || entry.hash != hash
                    }) {
                        log.add(LogEntry {
                            input: input.clone(),
                            modified_time,
                            hash,
                        });
                        inputs_changed = true;
                    }
                }
            }

            // Execute command if inputs have changed
            if inputs_changed {
                // Create output directories
                for output in &task_outputs {
                    if let Some(parent) = Path::new(output).parent() {
                        fs::create_dir_all(parent).unwrap_or_else(|e| {
                            eprintln!("Can't create output directory: {} {}", parent.display(), e);
                            exit(1)
                        });
                    }
                }

                // Execute command
                println!("{}", task_command);
                #[cfg(target_os = "windows")]
                {
                    let parts = task_command.split(' ').collect::<Vec<String>>();
                    let status = std::process::Command::new(parts[0])
                        .args(&parts[1..])
                        .status()
                        .unwrap_or_else(|_| {
                            eprintln!("Failed to execute command: {}", task_command);
                            exit(1)
                        });
                    if !status.success() {
                        panic!("Command failed: {}", task_command);
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&task_command)
                        .status()
                        .unwrap_or_else(|_| {
                            eprintln!("Failed to execute command: {}", task_command);
                            exit(1)
                        });
                    if !status.success() {
                        panic!("Command failed: {}", task_command);
                    }
                }

                // Write .stamp files to output dirs
                for output in &task_outputs {
                    let metadata = fs::metadata(output).unwrap_or_else(|_| {
                        eprintln!("Can't open output file: {}", output);
                        exit(1)
                    });
                    if metadata.is_dir() {
                        let stamp_path = format!("{}/.stamp", output);
                        File::create(&stamp_path).unwrap_or_else(|e| {
                            eprintln!("Can't create stamp file: {} {}", &stamp_path, e);
                            exit(1)
                        });
                    }
                }
            }

            // Mark task as done
            {
                let mut done_ids = done_task_ids.lock().expect("Could not lock mutex");
                done_ids.push(task_id);
            }
        });
    }
}
