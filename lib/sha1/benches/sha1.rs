/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! SHA-1 throughput and call overhead benchmarks.

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sha1::Sha1;

fn hash(c: &mut Criterion) {
    let backend = if cfg!(all(
        feature = "disable-simd",
        not(feature = "disable-software")
    )) {
        "software"
    } else if cfg!(all(
        feature = "disable-software",
        not(feature = "disable-simd")
    )) {
        "simd"
    } else {
        "auto"
    };
    let mut group = c.benchmark_group(format!("sha1_{backend}"));
    for size in [0, 55, 56, 63, 64, 65, 1024, 1024 * 1024] {
        let input = vec![0x5a; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("digest", size), &input, |b, input| {
            b.iter(|| black_box(Sha1::digest(black_box(input))));
        });
    }
    group.finish();

    let input = vec![0x5a; 1024 * 1024];
    let mut group = c.benchmark_group(format!("sha1_{backend}_update"));
    group.throughput(Throughput::Bytes(input.len() as u64));
    for chunk_size in [64, 1024, 16 * 1024] {
        group.bench_with_input(BenchmarkId::new("chunk", chunk_size), &input, |b, input| {
            b.iter(|| {
                let mut hasher = Sha1::new();
                for chunk in black_box(input).chunks(chunk_size) {
                    hasher.update(chunk);
                }
                black_box(hasher.finalize());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, hash);
criterion_main!(benches);
