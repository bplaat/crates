/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;
use std::process::{Command, exit};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::{env, fs, thread};

use sha1::{Digest, Sha1};
use threadpool::ThreadPool;

use crate::log::{Log, LogEntry};

// MARK: Task
#[derive(Debug, Clone)]
pub(crate) struct Task {
    id: usize,
    action: TaskAction,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum TaskAction {
    Phony(String),
    Copy(String, String),
    Command(String),
}

impl Task {
    fn have_inputs_change(&self, log: &Log) -> bool {
        // Check if inputs have changed
        for input in &self.inputs {
            // Get input modified time
            let metadata = match fs::metadata(input) {
                Ok(metadata) => metadata,
                Err(_) => return true,
            };
            let mtime = metadata
                .modified()
                .expect("Failed to get modified time")
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards");

            // Get input hash
            let hash = if !metadata.is_dir() {
                // Calculate input hash
                let buffer = fs::read(input).unwrap_or_else(|_| {
                    eprintln!("Can't read input file: {input}");
                    exit(1)
                });
                if !buffer.is_empty() {
                    let mut hasher = Sha1::new();
                    hasher.update(buffer);
                    Some(hasher.finalize().to_vec())
                } else {
                    None
                }
            } else {
                None
            };

            // Check if the input has changed
            if log
                .get(input)
                .is_none_or(|entry| entry.mtime != mtime || entry.hash != hash)
            {
                return true;
            }
        }

        // Check if outputs are missing
        for output in &self.outputs {
            if !Path::new(output).exists() {
                return true;
            }
        }

        false
    }

    fn execute(
        &self,
        log: Arc<Mutex<Log>>,
        task_counter: Arc<AtomicUsize>,
        total_tasks: usize,
        pretty_print: bool,
    ) {
        // Update log entries of inputs
        {
            let mut log: std::sync::MutexGuard<'_, Log> = log.lock().expect("Could not lock mutex");
            for input in &self.inputs {
                // Get input modified time
                let metadata = fs::metadata(input).unwrap_or_else(|_| {
                    eprintln!("Can't open input file: {input}");
                    exit(1)
                });
                let mtime = metadata
                    .modified()
                    .expect("Failed to get modified time")
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards");

                // Get input hash
                let hash = if !metadata.is_dir() {
                    // Calculate input hash
                    let buffer = fs::read(input).unwrap_or_else(|_| {
                        eprintln!("Can't read input file: {input}");
                        exit(1)
                    });
                    if !buffer.is_empty() {
                        let mut hasher = Sha1::new();
                        hasher.update(buffer);
                        Some(hasher.finalize().to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if log
                    .get(input)
                    .is_none_or(|entry| entry.mtime != mtime || entry.hash != hash)
                {
                    log.add(LogEntry {
                        path: input.clone(),
                        mtime,
                        hash,
                    });
                }
            }
        }

        // Create output directories
        for output in &self.outputs {
            if let Some(parent) = Path::new(output).parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    eprintln!("Can't create output directory: {} {}", parent.display(), e);
                    exit(1)
                });
            }
        }

        // Execute command
        self.action.execute(task_counter, total_tasks, pretty_print);

        // Update log entries of output dirs
        {
            let mut log = log.lock().expect("Could not lock mutex");
            for output in &self.outputs {
                let metadata = fs::metadata(output).unwrap_or_else(|_| {
                    eprintln!("Can't open output file: {output}");
                    exit(1)
                });
                if metadata.is_dir() {
                    log.add(LogEntry {
                        path: output.clone(),
                        mtime: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("Time went backwards")
                            - Duration::from_nanos(1),
                        hash: None,
                    });
                }
            }
        }
    }
}

impl TaskAction {
    fn execute(&self, task_counter: Arc<AtomicUsize>, total_tasks: usize, pretty_print: bool) {
        let line = match self {
            TaskAction::Phony(dest) => dest.clone(),
            TaskAction::Copy(src, dst) => {
                fs::copy(src, dst).unwrap_or_else(|_| {
                    eprintln!("Failed to copy {src} to {dst}");
                    exit(1)
                });
                format!("cp {src} {dst}")
            }
            TaskAction::Command(command) => {
                let status = if cfg!(windows) {
                    if command.contains("&&") {
                        let mut parts = command.split(' ').collect::<Vec<_>>();
                        parts.insert(0, "/c");
                        Command::new("cmd").args(parts).status()
                    } else {
                        let parts = command.split(' ').collect::<Vec<_>>();
                        Command::new(parts[0]).args(&parts[1..]).status()
                    }
                } else {
                    Command::new("sh").arg("-c").arg(command).status()
                }
                .unwrap_or_else(|_| {
                    eprintln!("Failed to execute command: {command}");
                    exit(1)
                });
                if !status.success() {
                    eprintln!("Command failed: {command}");
                    exit(1);
                }
                command.clone()
            }
        };
        let current_task = task_counter.fetch_add(1, Ordering::SeqCst);
        let line = format!("[{current_task}/{total_tasks}] {line}");

        if pretty_print {
            let term_width = terminal_size::terminal_size()
                .map(|(w, _)| w.0 as usize)
                .unwrap_or(80);
            println!(
                "\x1B[1A\x1B[2K{}",
                if line.len() > term_width {
                    format!("{}...", &line[..term_width - 3])
                } else {
                    line.to_string()
                }
            );
        } else {
            println!("{line}");
        }
    }
}

// MARK: ExecutorBuilder
pub(crate) struct ExecutorBuilder {
    tasks_id_counter: usize,
    tasks: Vec<Task>,
}

impl ExecutorBuilder {
    pub(crate) fn new() -> Self {
        Self {
            tasks_id_counter: 0,
            tasks: Vec::new(),
        }
    }

    fn add_task(&mut self, action: TaskAction, inputs: Vec<String>, outputs: Vec<String>) {
        if !self.tasks.iter().any(|task| task.outputs == outputs) {
            self.tasks.push(Task {
                id: self.tasks_id_counter,
                action,
                inputs,
                outputs,
            });
            self.tasks_id_counter += 1;
        }
    }

    pub(crate) fn add_task_phony(&mut self, inputs: Vec<String>, outputs: Vec<String>) {
        self.add_task(TaskAction::Phony(outputs.join(" ")), inputs, outputs);
    }

    pub(crate) fn add_task_cmd(
        &mut self,
        command: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) {
        self.add_task(TaskAction::Command(command), inputs, outputs);
    }

    pub(crate) fn add_task_cp(&mut self, src: String, dst: String) {
        self.add_task(
            TaskAction::Copy(src.clone(), dst.clone()),
            vec![src],
            vec![dst],
        );
    }

    pub(crate) fn build(self, log_path: &str) -> Executor {
        Executor::new(self.tasks, log_path)
    }
}

// MARK: Executor
pub(crate) struct Executor {
    log: Arc<Mutex<Log>>,
    tasks: Vec<Task>,
}

impl Executor {
    fn new(tasks: Vec<Task>, log_path: &str) -> Self {
        fn visit_task(
            task: &Task,
            all_tasks: &[Task],
            new_tasks: &mut Vec<Task>,
            log: &Log,
        ) -> bool {
            if new_tasks.iter().any(|t| t.id == task.id) {
                return false;
            }

            let mut inputs_changed = task.have_inputs_change(log);
            for input in &task.inputs {
                for other_task in all_tasks {
                    if other_task.outputs.contains(input) {
                        inputs_changed |= visit_task(other_task, all_tasks, new_tasks, log);
                    }
                }
            }

            if inputs_changed {
                new_tasks.push(task.clone());
            }
            inputs_changed
        }

        // Create new task tree with all needed tasks
        let log = Log::new(log_path);
        let mut new_tasks = Vec::new();
        let last_task = tasks.last().expect("No tasks to execute");
        visit_task(last_task, &tasks, &mut new_tasks, &log);

        Self {
            log: Arc::new(Mutex::new(log)),
            tasks: new_tasks,
        }
    }

    pub(crate) fn total_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub(crate) fn execute(&mut self, verbose: bool, thread_count: Option<usize>) {
        // Start execution if there is a last task
        if let Some(last_task) = self.tasks.last() {
            // Print task tree
            if verbose {
                println!("{:#?}", self.tasks);
            }

            let pretty_print = !verbose && env::var("NO_COLOR").is_err() && env::var("CI").is_err();
            if pretty_print {
                println!();
            }

            let pool = ThreadPool::new(
                thread_count
                    .unwrap_or_else(|| thread::available_parallelism().map_or(1, |n| n.get())),
            );
            self.execute_task(
                last_task,
                &pool,
                Arc::new(Mutex::new(Vec::new())),
                Arc::new(Mutex::new(Vec::new())),
                Arc::new(AtomicUsize::new(1)),
                pretty_print,
            );
            pool.join();
        }
    }

    fn execute_task(
        &self,
        task: &Task,
        pool: &ThreadPool,
        scheduled_task_ids: Arc<Mutex<Vec<usize>>>,
        done_task_ids: Arc<Mutex<Vec<usize>>>,
        task_counter: Arc<AtomicUsize>,
        pretty_print: bool,
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
                        task_counter.clone(),
                        pretty_print,
                    );
                }
            }
        }

        let task = task.clone();
        let log = self.log.clone();
        let task_counter: Arc<AtomicUsize> = task_counter.clone();
        let total_tasks = self.tasks.len();
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

            // Execute task
            task.execute(log, task_counter, total_tasks, pretty_print);

            // Mark task as done
            {
                let mut done_ids = done_task_ids.lock().expect("Could not lock mutex");
                done_ids.push(task.id);
            }
        });
    }
}
