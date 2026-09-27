/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! PBKDF2-HMAC-SHA256 iteration and output length benchmarks.

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn derive_key(c: &mut Criterion) {
    let password = b"benchmark password";
    let salt = b"benchmark salt!!";
    let mut group = c.benchmark_group("pbkdf2_sha256_iterations");
    for iterations in [1_000, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("rounds", iterations),
            &iterations,
            |b, &rounds| {
                b.iter(|| {
                    black_box(pbkdf2::pbkdf2_hmac_sha256(
                        black_box(password),
                        black_box(salt),
                        rounds,
                        32,
                    ))
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("pbkdf2_sha256_output");
    for length in [32, 64] {
        group.bench_with_input(BenchmarkId::new("bytes", length), &length, |b, &length| {
            b.iter(|| {
                black_box(pbkdf2::pbkdf2_hmac_sha256(
                    black_box(password),
                    black_box(salt),
                    10_000,
                    length,
                ))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, derive_key);
criterion_main!(benches);
