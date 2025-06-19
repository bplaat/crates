/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;
use std::process::{Command, exit};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{fs, thread};

use threadpool::ThreadPool;
use uuid::Uuid;

use crate::log::{Log, LogEntry};
use crate::sha1::sha1;

// MARK: Task
#[derive(Debug, Clone)]
pub(crate) enum TaskAction {
    Phony,
    Copy(String, String),
    Command(String),
}

impl TaskAction {
    fn execute(&self) {
        match self {
            TaskAction::Phony => {}
            TaskAction::Copy(src, dst) => {
                println!("cp {} {}", src, dst);
                fs::copy(src, dst).unwrap_or_else(|_| {
                    eprintln!("Failed to copy {} to {}", src, dst);
                    exit(1)
                });
            }
            TaskAction::Command(command) => {
                println!("{}", command);
                let status = if cfg!(windows) {
                    let parts = command.split(' ').collect::<Vec<_>>();
                    Command::new(parts[0]).args(&parts[1..]).status()
                } else {
                    Command::new("sh").arg("-c").arg(command).status()
                }
                .unwrap_or_else(|_| {
                    eprintln!("Failed to execute command: {}", command);
                    exit(1)
                });
                if !status.success() {
                    panic!("Command failed: {}", command);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Task {
    id: Uuid,
    action: TaskAction,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

// MARK: Executor
#[derive(Debug)]
pub(crate) struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub(crate) fn add_task(
        &mut self,
        action: TaskAction,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) {
        self.tasks.push(Task {
            id: Uuid::new_v4(),
            action,
            inputs,
            outputs,
        });
    }

    pub(crate) fn add_task_phony(&mut self, inputs: Vec<String>, outputs: Vec<String>) {
        self.add_task(TaskAction::Phony, inputs, outputs);
    }

    pub(crate) fn add_task_cmd(
        &mut self,
        action: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) {
        self.add_task(TaskAction::Command(action), inputs, outputs);
    }

    pub(crate) fn add_task_cp(&mut self, src: String, dst: String) {
        self.add_task(
            TaskAction::Copy(src.clone(), dst.clone()),
            vec![src],
            vec![dst],
        );
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
        scheduled_task_ids: Arc<Mutex<Vec<Uuid>>>,
        done_task_ids: Arc<Mutex<Vec<Uuid>>>,
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

        let task = task.clone();
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
            for input in &task.inputs {
                // Get input modified time
                let metadata = fs::metadata(input).unwrap_or_else(|_| {
                    eprintln!("Can't open input file: {}", input);
                    exit(1)
                });
                let modified_time = metadata
                    .modified()
                    .expect("Failed to get modified time")
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                // Get input hash
                let is_dir = fs::metadata(input)
                    .map(|metadata| metadata.is_dir())
                    .expect("Failed to check input metadata");
                let hash = if is_dir {
                    None
                } else {
                    // Calculate input hash
                    let buffer = fs::read(input).unwrap_or_else(|_| {
                        eprintln!("Can't read input file: {}", input);
                        exit(1)
                    });
                    if !buffer.is_empty() {
                        Some(sha1(&buffer))
                    } else {
                        None
                    }
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
                for output in &task.outputs {
                    if let Some(parent) = Path::new(output).parent() {
                        fs::create_dir_all(parent).unwrap_or_else(|e| {
                            eprintln!("Can't create output directory: {} {}", parent.display(), e);
                            exit(1)
                        });
                    }
                }

                // Execute command
                task.action.execute();

                // Update log entries of output dirs
                {
                    let mut log = log.lock().expect("Could not lock mutex");
                    for output in &task.outputs {
                        let metadata = fs::metadata(output).unwrap_or_else(|_| {
                            eprintln!("Can't open output file: {}", output);
                            exit(1)
                        });
                        if metadata.is_dir() {
                            log.add(LogEntry {
                                input: output.clone(),
                                modified_time: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .expect("Time went backwards")
                                    .as_secs()
                                    - 1,
                                hash: None,
                            });
                        }
                    }
                }
            }

            // Mark task as done
            {
                let mut done_ids = done_task_ids.lock().expect("Could not lock mutex");
                done_ids.push(task.id);
            }
        });
    }
}
