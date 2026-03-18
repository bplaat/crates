/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::process::{Command, exit};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime};
use std::{fs, thread};

use sha1::Sha1;
use threadpool::ThreadPool;

use crate::log::{Log, LogEntry};

// MARK: File info helper
/// Returns (mtime, optional SHA-1 hash) for a file path.
/// Centralises the computation that is needed both for change detection and
/// for writing log entries -- avoids reading / hashing each file twice.
fn file_info(path: &str) -> (Duration, Option<Vec<u8>>) {
    let metadata = fs::metadata(path).unwrap_or_else(|_| {
        eprintln!("can't stat file: {path}");
        exit(1)
    });
    let mtime = metadata
        .modified()
        .expect("failed to get modified time")
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time went backwards");
    let hash = if !metadata.is_dir() {
        let buf = fs::read(path).unwrap_or_else(|_| {
            eprintln!("can't read file: {path}");
            exit(1)
        });
        if buf.is_empty() {
            None
        } else {
            Some(Sha1::digest(buf).to_vec())
        }
    } else {
        None
    };
    (mtime, hash)
}

// MARK: Task
/// An opaque build task managed by [`ExecutorBuilder`] and [`Executor`].
#[derive(Debug, Clone)]
pub struct Task {
    id: usize,
    action: TaskAction,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

/// The action a [`Task`] performs when executed.
#[derive(Debug, Clone)]
pub enum TaskAction {
    /// No-op marker task; the label is the joined outputs string.
    Phony(String),
    /// Copy a file from `src` to `dst`.
    Copy(String, String),
    /// Run a shell command.
    Command(String),
    /// Send a line over a Unix domain socket and wait for an exit-code response.
    /// Only available on Unix; exits with an error on other platforms.
    SendMsg(String, String),
    /// Run several actions sequentially and display them joined with ` && `.
    Multiple(Vec<TaskAction>),
}

impl Task {
    fn have_inputs_changed(&self, log: &Log) -> bool {
        for input in &self.inputs {
            let metadata = match fs::metadata(input) {
                Ok(m) => m,
                Err(_) => return true,
            };
            let mtime = metadata
                .modified()
                .expect("failed to get modified time")
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time went backwards");
            let hash = if !metadata.is_dir() {
                let buf = fs::read(input).unwrap_or_else(|_| {
                    eprintln!("can't read input file: {input}");
                    exit(1)
                });
                if buf.is_empty() { None } else { Some(Sha1::digest(buf).to_vec()) }
            } else {
                None
            };
            if log
                .get(input)
                .is_none_or(|e| e.mtime != mtime || e.hash != hash)
            {
                return true;
            }
        }
        for output in &self.outputs {
            if !Path::new(output).exists() {
                return true;
            }
        }
        false
    }

    fn run(
        &self,
        log: Arc<Mutex<Log>>,
        task_counter: Arc<AtomicUsize>,
        total_tasks: usize,
    ) -> (usize, usize, String) {
        // Record input state (using file_info helper -- no duplicate read/hash).
        {
            let mut log = log.lock().expect("could not lock mutex");
            for input in &self.inputs {
                let (mtime, hash) = file_info(input);
                if log.get(input).is_none_or(|e| e.mtime != mtime || e.hash != hash) {
                    log.add(LogEntry { path: input.clone(), mtime, hash });
                }
            }
        }

        // Ensure output parent directories exist.
        for output in &self.outputs {
            if let Some(parent) = Path::new(output).parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    eprintln!("can't create output directory: {} {e}", parent.display());
                    exit(1)
                });
            }
        }

        // Execute action and capture its label.
        let label = self.action.run();
        let current = task_counter.fetch_add(1, Ordering::SeqCst);

        // Update mtime bookkeeping for output directories.
        {
            let mut log = log.lock().expect("could not lock mutex");
            for output in &self.outputs {
                if fs::metadata(output)
                    .map(|m| m.is_dir())
                    .unwrap_or(false)
                {
                    log.add(LogEntry {
                        path: output.clone(),
                        mtime: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("time went backwards")
                            - Duration::from_nanos(1),
                        hash: None,
                    });
                }
            }
        }

        (current, total_tasks, label)
    }
}

impl TaskAction {
    fn run(&self) -> String {
        match self {
            TaskAction::Phony(label) => label.clone(),
            TaskAction::Copy(src, dst) => {
                fs::copy(src, dst).unwrap_or_else(|_| {
                    eprintln!("failed to copy {src} to {dst}");
                    exit(1)
                });
                format!("cp {src} {dst}")
            }
            TaskAction::Command(command) => {
                let status = if cfg!(windows) {
                    let parts = command.split(' ').collect::<Vec<_>>();
                    Command::new(parts[0]).args(&parts[1..]).status()
                } else {
                    Command::new("sh").arg("-c").arg(command).status()
                }
                .unwrap_or_else(|_| {
                    eprintln!("failed to execute command: {command}");
                    exit(1)
                });
                if !status.success() {
                    eprintln!("command failed: {command}");
                    exit(1);
                }
                command.clone()
            }
            TaskAction::SendMsg(socket_path, line) => {
                #[cfg(unix)]
                {
                    use std::io::{Read, Write};
                    let mut stream = std::os::unix::net::UnixStream::connect(socket_path)
                        .expect("failed to connect to socket");
                    _ = stream.write_all(line.as_bytes());
                    _ = stream.flush();
                    let mut response = Vec::new();
                    _ = stream.read_to_end(&mut response);
                    let read_line = String::from_utf8_lossy(&response);
                    let (exit_code, stderr) =
                        read_line.split_once('\n').expect("invalid socket response");
                    if exit_code.parse::<i32>().expect("invalid exit code") != 0 {
                        eprintln!("command failed: {line}\n{stderr}");
                        exit(1);
                    }
                    line.clone()
                }
                #[cfg(not(unix))]
                {
                    eprintln!("SendMsg is only supported on Unix systems");
                    exit(1);
                }
            }
            TaskAction::Multiple(actions) => actions
                .iter()
                .map(|a| a.run())
                .collect::<Vec<_>>()
                .join(" && "),
        }
    }
}

// MARK: ExecutorBuilder
/// Accumulates tasks and produces an [`Executor`] via [`ExecutorBuilder::build`].
pub struct ExecutorBuilder {
    id_counter: usize,
    tasks: Vec<Task>,
    // Fix #9: O(1) dedup check keyed on outputs.
    outputs_seen: HashSet<Vec<String>>,
}

impl ExecutorBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            tasks: Vec::new(),
            outputs_seen: HashSet::new(),
        }
    }

    /// Add a task with an arbitrary action. Duplicate tasks (same outputs) are silently ignored.
    pub fn add_task(&mut self, action: TaskAction, inputs: Vec<String>, outputs: Vec<String>) {
        if self.outputs_seen.insert(outputs.clone()) {
            self.tasks.push(Task {
                id: self.id_counter,
                action,
                inputs,
                outputs,
            });
            self.id_counter += 1;
        }
    }

    /// Add a phony (no-op) task.
    pub fn add_task_phony(&mut self, inputs: Vec<String>, outputs: Vec<String>) {
        self.add_task(TaskAction::Phony(outputs.join(" ")), inputs, outputs);
    }

    /// Add a shell command task.
    pub fn add_task_cmd(&mut self, command: String, inputs: Vec<String>, outputs: Vec<String>) {
        self.add_task(TaskAction::Command(command), inputs, outputs);
    }

    /// Add a file-copy task.
    pub fn add_task_cp(&mut self, src: String, dst: String) {
        self.add_task(
            TaskAction::Copy(src.clone(), dst.clone()),
            vec![src],
            vec![dst],
        );
    }

    /// Finalise the task graph and return an [`Executor`] ready to run.
    pub fn build(self, log_path: &str) -> Executor {
        Executor::new(self.tasks, log_path)
    }
}

impl Default for ExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// MARK: Cycle detection via Kahn's algorithm (Fix #3)
/// Returns `Ok(topological_order)` or `Err(cycle_task_ids)`.
fn topological_sort(tasks: &[Task]) -> Result<Vec<usize>, Vec<usize>> {
    // Build adjacency: for each task, which task ids does it depend on?
    let mut in_degree: Vec<usize> = vec![0; tasks.len()];
    // id_to_index map
    let id_to_idx: std::collections::HashMap<usize, usize> =
        tasks.iter().enumerate().map(|(i, t)| (t.id, i)).collect();

    // edges[i] = list of task indices that depend on task i (i.e. i is a prerequisite for them)
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); tasks.len()];

    for (idx, task) in tasks.iter().enumerate() {
        for input in &task.inputs {
            for (other_idx, other_task) in tasks.iter().enumerate() {
                if other_task.outputs.contains(input) {
                    // task depends on other_task
                    edges[other_idx].push(idx);
                    in_degree[idx] += 1;
                }
            }
        }
    }

    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|&(_, d)| *d == 0)
        .map(|(i, _)| i)
        .collect();

    let mut order = Vec::with_capacity(tasks.len());
    while let Some(idx) = queue.pop_front() {
        order.push(idx);
        for &dependent_idx in &edges[idx].clone() {
            in_degree[dependent_idx] -= 1;
            if in_degree[dependent_idx] == 0 {
                queue.push_back(dependent_idx);
            }
        }
    }

    if order.len() == tasks.len() {
        // No cycle -- return task ids in topological order.
        Ok(order.into_iter().map(|i| tasks[i].id).collect())
    } else {
        // Remaining tasks with in_degree > 0 are part of a cycle.
        let cycle: Vec<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|&(_, d)| *d > 0)
            .map(|(i, _)| tasks[i].id)
            .collect();
        // Suppress the unused variable warning for id_to_idx in the success path.
        let _ = id_to_idx;
        Err(cycle)
    }
}

fn abort_on_cycle(cycle_ids: &[usize], tasks: &[Task]) {
    eprintln!("circular dependency detected in task graph!\n");
    eprintln!("tasks involved in the cycle:");
    for task in tasks.iter().filter(|t| cycle_ids.contains(&t.id)) {
        eprintln!(
            "  task {}: inputs={:?} outputs={:?}",
            task.id, task.inputs, task.outputs
        );
    }
    exit(1);
}

// MARK: Executor
/// Executes a filtered subset of tasks in dependency order using a thread pool.
pub struct Executor {
    log: Arc<Mutex<Log>>,
    tasks: Vec<Task>,
}

type ProgressFn = Arc<dyn Fn(usize, usize, &str) + Send + Sync + 'static>;

impl Executor {
    fn new(tasks: Vec<Task>, log_path: &str) -> Self {
        // Fix #3: single-pass cycle detection via Kahn's topological sort.
        let topo = match topological_sort(&tasks) {
            Ok(order) => order,
            Err(cycle_ids) => {
                abort_on_cycle(&cycle_ids, &tasks);
                unreachable!()
            }
        };

        let log = Log::new(log_path);

        // Filter to only the tasks whose inputs have changed (or whose outputs
        // are missing), preserving topological order.  Unlike the original
        // implementation this does *not* assume a single last task as the only
        // root -- every task is a potential root.
        fn visit(
            task_id: usize,
            tasks: &[Task],
            log: &Log,
            needed: &mut Vec<Task>,
            visited: &mut HashSet<usize>,
        ) -> bool {
            if visited.contains(&task_id) {
                return needed.iter().any(|t| t.id == task_id);
            }
            visited.insert(task_id);

            let task = tasks.iter().find(|t| t.id == task_id).expect("task not found");
            let mut inputs_changed = task.have_inputs_changed(log);

            for input in &task.inputs {
                for dep in tasks {
                    if dep.outputs.contains(input) {
                        inputs_changed |= visit(dep.id, tasks, log, needed, visited);
                    }
                }
            }

            if inputs_changed && !needed.iter().any(|t| t.id == task_id) {
                needed.push(task.clone());
            }
            inputs_changed
        }

        // Walk every task that has no dependents (i.e. is a "leaf" in the
        // build graph) so all required work is discovered, not just the chain
        // reachable from the last task.
        let dependent_ids: HashSet<usize> = tasks
            .iter()
            .flat_map(|t| t.inputs.iter())
            .flat_map(|input| tasks.iter().filter(move |t| t.outputs.contains(input)).map(|t| t.id))
            .collect();

        let roots: Vec<usize> = tasks
            .iter()
            .filter(|t| !dependent_ids.contains(&t.id))
            .map(|t| t.id)
            .collect();

        let mut visited = HashSet::new();
        let mut needed: Vec<Task> = Vec::new();
        for root_id in roots {
            visit(root_id, &tasks, &log, &mut needed, &mut visited);
        }

        // Sort the needed tasks into topological execution order.
        let topo_pos: std::collections::HashMap<usize, usize> =
            topo.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        needed.sort_by_key(|t| topo_pos.get(&t.id).copied().unwrap_or(usize::MAX));

        Self {
            log: Arc::new(Mutex::new(log)),
            tasks: needed,
        }
    }

    /// Number of tasks that will actually be executed (after incremental filtering).
    pub fn total_tasks(&self) -> usize {
        self.tasks.len()
    }

    /// Execute all pending tasks using up to `thread_count` threads (or all
    /// available CPU cores when `None`).
    ///
    /// `on_progress` is called from worker threads after each task completes
    /// with `(current, total, label)` — the consumer is responsible for all
    /// display logic.
    pub fn execute(
        &mut self,
        thread_count: Option<usize>,
        on_progress: impl Fn(usize, usize, &str) + Send + Sync + 'static,
    ) {
        if let Some(last_task) = self.tasks.last() {
            let pool = ThreadPool::new(
                thread_count.unwrap_or_else(|| thread::available_parallelism().map_or(1, |n| n.get())),
            );

            let done: Arc<(Mutex<HashSet<usize>>, Condvar)> =
                Arc::new((Mutex::new(HashSet::new()), Condvar::new()));
            let scheduled: Arc<Mutex<HashSet<usize>>> = Arc::new(Mutex::new(HashSet::new()));
            let task_counter = Arc::new(AtomicUsize::new(1));
            let on_progress: ProgressFn = Arc::new(on_progress);

            self.schedule_task(
                last_task,
                &pool,
                scheduled.clone(),
                done.clone(),
                task_counter.clone(),
                on_progress,
            );
            pool.join();
        }
    }

    fn schedule_task(
        &self,
        task: &Task,
        pool: &ThreadPool,
        scheduled: Arc<Mutex<HashSet<usize>>>,
        done: Arc<(Mutex<HashSet<usize>>, Condvar)>,
        task_counter: Arc<AtomicUsize>,
        on_progress: ProgressFn,
    ) {
        {
            let mut sched = scheduled.lock().expect("could not lock mutex");
            if sched.contains(&task.id) {
                return;
            }
            sched.insert(task.id);
        }

        // Schedule dependencies first, collecting their ids.
        let mut dep_ids: Vec<usize> = Vec::new();
        for input in &task.inputs {
            for dep in &self.tasks {
                if dep.outputs.contains(input) {
                    dep_ids.push(dep.id);
                    self.schedule_task(
                        dep,
                        pool,
                        scheduled.clone(),
                        done.clone(),
                        task_counter.clone(),
                        on_progress.clone(),
                    );
                }
            }
        }

        let task = task.clone();
        let log = self.log.clone();
        let total_tasks = self.tasks.len();
        pool.execute(move || {
            if !dep_ids.is_empty() {
                let (lock, cvar) = &*done;
                let mut finished = lock.lock().expect("could not lock mutex");
                while !dep_ids.iter().all(|id| finished.contains(id)) {
                    finished = cvar.wait(finished).expect("condvar wait failed");
                }
            }

            let (current, total, label) = task.run(log, task_counter, total_tasks);
            on_progress(current, total, &label);

            let (lock, cvar) = &*done;
            {
                let mut finished = lock.lock().expect("could not lock mutex");
                finished.insert(task.id);
            }
            cvar.notify_all();
        });
    }
}
