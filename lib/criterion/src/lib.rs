/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::collections::BTreeMap;
pub use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fmt, fs};

/// Identifies one benchmark within a group.
pub struct BenchmarkId {
    function: String,
    parameter: String,
}

impl BenchmarkId {
    /// Create a benchmark ID from a function name and parameter value.
    pub fn new(function: impl Into<String>, parameter: impl fmt::Display) -> Self {
        Self {
            function: function.into(),
            parameter: parameter.to_string(),
        }
    }
}

impl fmt::Display for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.function, self.parameter)
    }
}

/// Amount of work performed by one benchmark iteration.
#[derive(Clone, Copy)]
pub enum Throughput {
    /// Number of input bytes.
    Bytes(u64),
    /// Number of processed elements.
    Elements(u64),
}

/// Amount of setup data retained at once by batched benchmarks.
#[derive(Clone, Copy)]
pub enum BatchSize {
    /// Retain up to 1,000 small inputs per batch.
    SmallInput,
    /// Retain up to 10 large inputs per batch.
    LargeInput,
    /// Create one input at a time.
    PerIteration,
}

impl BatchSize {
    const fn count(self) -> u64 {
        match self {
            Self::SmallInput => 1_000,
            Self::LargeInput => 10,
            Self::PerIteration => 1,
        }
    }
}

/// Runs a benchmark closure for a calibrated number of iterations.
pub struct Bencher {
    iterations: u64,
    elapsed: Option<Duration>,
}

impl Bencher {
    /// Repeatedly run the closure and record its elapsed time.
    pub fn iter<F, R>(&mut self, mut routine: F)
    where
        F: FnMut() -> R,
    {
        let start = Instant::now();
        for _ in 0..self.iterations {
            black_box(routine());
        }
        self.elapsed = Some(start.elapsed());
    }

    /// Run setup outside the timer and consume each prepared input inside it.
    pub fn iter_batched<I, R>(
        &mut self,
        mut setup: impl FnMut() -> I,
        mut routine: impl FnMut(I) -> R,
        batch_size: BatchSize,
    ) {
        let mut remaining = self.iterations;
        let mut elapsed = Duration::ZERO;
        while remaining > 0 {
            let count = remaining.min(batch_size.count());
            let inputs = (0..count).map(|_| setup()).collect::<Vec<_>>();
            let start = Instant::now();
            for input in inputs {
                black_box(routine(input));
            }
            elapsed += start.elapsed();
            remaining -= count;
        }
        self.elapsed = Some(elapsed);
    }

    /// Run setup outside the timer and mutate each prepared input inside it.
    pub fn iter_batched_ref<I, R>(
        &mut self,
        mut setup: impl FnMut() -> I,
        mut routine: impl FnMut(&mut I) -> R,
        batch_size: BatchSize,
    ) {
        let mut remaining = self.iterations;
        let mut elapsed = Duration::ZERO;
        while remaining > 0 {
            let count = remaining.min(batch_size.count());
            let mut inputs = (0..count).map(|_| setup()).collect::<Vec<_>>();
            let start = Instant::now();
            for input in &mut inputs {
                black_box(routine(input));
            }
            elapsed += start.elapsed();
            remaining -= count;
        }
        self.elapsed = Some(elapsed);
    }

    /// Record a duration measured by a custom routine that runs `iterations` times.
    pub fn iter_custom(&mut self, routine: impl FnOnce(u64) -> Duration) {
        self.elapsed = Some(routine(self.iterations));
    }
}

struct Options {
    sample_size: usize,
    measurement_time: Duration,
    warm_up_time: Duration,
    filter: Option<String>,
    baseline: Option<PathBuf>,
    save_baseline: Option<PathBuf>,
    test: bool,
    list: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sample_size: 15,
            measurement_time: Duration::from_secs(3),
            warm_up_time: Duration::from_millis(100),
            filter: None,
            baseline: None,
            save_baseline: None,
            test: false,
            list: false,
        }
    }
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--sample-size" => {
                    options.sample_size = next_value(&mut args, &arg)?
                        .parse()
                        .map_err(|_| "invalid sample size".to_owned())?;
                    if options.sample_size == 0 {
                        return Err("sample size must be positive".to_owned());
                    }
                }
                "--measurement-time" => {
                    options.measurement_time = parse_duration(next_value(&mut args, &arg)?)?;
                }
                "--warm-up-time" => {
                    options.warm_up_time = parse_duration(next_value(&mut args, &arg)?)?;
                }
                "--baseline" => options.baseline = Some(next_value(&mut args, &arg)?.into()),
                "--save-baseline" => {
                    options.save_baseline = Some(next_value(&mut args, &arg)?.into());
                }
                "--test" => options.test = true,
                "--list" => options.list = true,
                "--bench" | "--nocapture" => {}
                _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
                _ => options.filter = Some(arg),
            }
        }
        Ok(options)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}"))
}

fn parse_duration(value: String) -> Result<Duration, String> {
    let seconds: f64 = value.parse().map_err(|_| "invalid duration".to_owned())?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("duration must be positive and finite".to_owned());
    }
    Ok(Duration::from_secs_f64(seconds))
}

/// Configures and collects benchmark results.
pub struct Criterion {
    options: Options,
    baseline: BTreeMap<String, f64>,
    results: BTreeMap<String, f64>,
}

impl Default for Criterion {
    fn default() -> Self {
        let options = Options::parse(std::env::args().skip(1))
            .unwrap_or_else(|error| panic!("benchmark options: {error}"));
        let baseline = options
            .baseline
            .as_ref()
            .map(read_baseline)
            .unwrap_or_default();
        Self {
            options,
            baseline,
            results: BTreeMap::new(),
        }
    }
}

impl Criterion {
    /// Set the number of timed samples.
    pub fn sample_size(mut self, size: usize) -> Self {
        assert!(size > 0, "sample size must be positive");
        self.options.sample_size = size;
        self
    }

    /// Set the total target measurement time across all samples.
    pub fn measurement_time(mut self, duration: Duration) -> Self {
        assert!(!duration.is_zero(), "measurement time must be positive");
        self.options.measurement_time = duration;
        self
    }

    /// Set the minimum warm-up time per benchmark.
    pub fn warm_up_time(mut self, duration: Duration) -> Self {
        assert!(!duration.is_zero(), "warm-up time must be positive");
        self.options.warm_up_time = duration;
        self
    }

    /// Benchmark one named routine without creating a group.
    pub fn bench_function(&mut self, name: &str, mut routine: impl FnMut(&mut Bencher)) {
        self.run(name, None, |iterations| {
            let mut bencher = Bencher {
                iterations,
                elapsed: None,
            };
            routine(&mut bencher);
            bencher
                .elapsed
                .expect("benchmark must call a Bencher method")
        });
    }

    /// Create a named benchmark group.
    pub fn benchmark_group(&mut self, name: impl Into<String>) -> BenchmarkGroup<'_> {
        BenchmarkGroup {
            criterion: self,
            name: name.into(),
            throughput: None,
        }
    }

    /// Write median times when `--save-baseline PATH` was provided.
    pub fn finish(self) {
        if let Some(path) = self.options.save_baseline {
            let mut output = String::new();
            for (name, median) in self.results {
                output.push_str(&format!("{name}\t{median}\n"));
            }
            fs::write(&path, output)
                .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        }
    }

    fn run(
        &mut self,
        name: &str,
        throughput: Option<Throughput>,
        mut routine: impl FnMut(u64) -> Duration,
    ) {
        if self
            .options
            .filter
            .as_ref()
            .is_some_and(|filter| !name.contains(filter))
        {
            return;
        }
        if self.options.list {
            println!("{name}");
            return;
        }
        if self.options.test {
            routine(1);
            println!("{name}: Success");
            return;
        }

        let mut iterations = 1u64;
        let warmup_time = loop {
            let elapsed = routine(iterations);
            if elapsed >= self.options.warm_up_time || iterations > u64::MAX / 2 {
                break elapsed;
            }
            iterations *= 2;
        };
        let sample_time =
            self.options.measurement_time.as_nanos() / self.options.sample_size as u128;
        iterations = ((u128::from(iterations) * sample_time) / warmup_time.as_nanos().max(1))
            .clamp(1, u128::from(u64::MAX)) as u64;

        let mut samples = Vec::with_capacity(self.options.sample_size);
        for _ in 0..self.options.sample_size {
            let elapsed = routine(iterations);
            samples.push(elapsed.as_nanos() as f64 / iterations as f64);
        }
        samples.sort_by(f64::total_cmp);
        let median = samples[samples.len() / 2];
        let low = samples[samples.len() / 10];
        let high = samples[samples.len() * 9 / 10];
        let comparison = self
            .baseline
            .get(name)
            .map(|previous| format!("  {:+.1}% vs baseline", (median / previous - 1.0) * 100.0));
        let rate = match throughput {
            Some(Throughput::Bytes(bytes)) if bytes > 0 => {
                format!("  {:.1} MiB/s", bytes as f64 * 1e9 / median / 1_048_576.0)
            }
            Some(Throughput::Elements(elements)) if elements > 0 => {
                format!("  {:.1} elements/s", elements as f64 * 1e9 / median)
            }
            _ => String::new(),
        };
        println!(
            "{name:38} {} (p10 {}, p90 {}){rate}{}",
            format_time(median),
            format_time(low),
            format_time(high),
            comparison.unwrap_or_default()
        );
        self.results.insert(name.to_owned(), median);
    }
}

fn format_time(nanoseconds: f64) -> String {
    if nanoseconds >= 1_000_000.0 {
        format!("{:.3} ms", nanoseconds / 1_000_000.0)
    } else if nanoseconds >= 1_000.0 {
        format!("{:.3} us", nanoseconds / 1_000.0)
    } else {
        format!("{nanoseconds:.1} ns")
    }
}

fn read_baseline(path: &PathBuf) -> BTreeMap<String, f64> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    contents
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once('\t')?;
            Some((name.to_owned(), value.parse().ok()?))
        })
        .collect()
}

/// Benchmarks related cases under one name.
pub struct BenchmarkGroup<'a> {
    criterion: &'a mut Criterion,
    name: String,
    throughput: Option<Throughput>,
}

impl BenchmarkGroup<'_> {
    /// Set the amount of work per iteration for subsequent benchmarks.
    pub const fn throughput(&mut self, throughput: Throughput) {
        self.throughput = Some(throughput);
    }

    /// Benchmark one named routine in the group.
    pub fn bench_function(&mut self, name: &str, mut routine: impl FnMut(&mut Bencher)) {
        let name = format!("{}/{name}", self.name);
        self.criterion.run(&name, self.throughput, |iterations| {
            let mut bencher = Bencher {
                iterations,
                elapsed: None,
            };
            routine(&mut bencher);
            bencher
                .elapsed
                .expect("benchmark must call a Bencher method")
        });
    }

    /// Benchmark a routine using a fixed input.
    pub fn bench_with_input<I>(
        &mut self,
        id: BenchmarkId,
        input: &I,
        mut routine: impl FnMut(&mut Bencher, &I),
    ) {
        let name = format!("{}/{id}", self.name);
        self.criterion.run(&name, self.throughput, |iterations| {
            let mut bencher = Bencher {
                iterations,
                elapsed: None,
            };
            routine(&mut bencher, input);
            bencher
                .elapsed
                .expect("benchmark must call a Bencher method")
        });
    }

    /// Finish the group.
    pub fn finish(self) {}
}

/// Define a function that runs one or more benchmark functions.
#[macro_export]
macro_rules! criterion_group {
    ($name:ident, $($function:path),+ $(,)?) => {
        pub fn $name(criterion: &mut $crate::Criterion) {
            $($function(criterion);)+
        }
    };
}

/// Generate a benchmark executable entry point.
#[macro_export]
macro_rules! criterion_main {
    ($($group:path),+ $(,)?) => {
        fn main() {
            let mut criterion = $crate::Criterion::default();
            $($group(&mut criterion);)+
            criterion.finish();
        }
    };
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn parses_options() {
        let options = Options::parse([
            "--sample-size".into(),
            "20".into(),
            "--measurement-time".into(),
            "2".into(),
            "--warm-up-time".into(),
            "0.5".into(),
            "digest".into(),
        ])
        .expect("valid options");
        assert_eq!(options.sample_size, 20);
        assert_eq!(options.measurement_time, Duration::from_secs(2));
        assert_eq!(options.warm_up_time, Duration::from_millis(500));
        assert_eq!(options.filter.as_deref(), Some("digest"));
    }

    #[test]
    fn rejects_bad_options() {
        assert!(Options::parse(["--sample-size".into(), "0".into()]).is_err());
        assert!(Options::parse(["--measurement-time".into(), "nan".into()]).is_err());
    }

    #[test]
    fn batched_benchmarks_run_all_iterations() {
        let setups = Cell::new(0);
        let operations = Cell::new(0);
        let mut bencher = Bencher {
            iterations: 23,
            elapsed: None,
        };
        bencher.iter_batched(
            || {
                setups.set(setups.get() + 1);
                0
            },
            |_| operations.set(operations.get() + 1),
            BatchSize::LargeInput,
        );
        assert_eq!(setups.get(), 23);
        assert_eq!(operations.get(), 23);
        assert!(bencher.elapsed.is_some());

        bencher.iter_batched_ref(
            || 0,
            |input| {
                *input += 1;
                operations.set(operations.get() + 1);
            },
            BatchSize::PerIteration,
        );
        assert_eq!(operations.get(), 46);
        assert!(bencher.elapsed.is_some());
    }
}
