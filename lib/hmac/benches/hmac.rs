/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! HMAC-SHA256 message size and key length benchmarks.

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sha2::Sha256;

fn hmac_sha256(c: &mut Criterion) {
    let key = [0x5a; 32];
    let mut group = c.benchmark_group("hmac_sha256_message");
    for size in [32, 1024, 1024 * 1024] {
        let message = vec![0xa5; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", size), &message, |b, message| {
            b.iter(|| black_box(hmac::hmac::<Sha256>(black_box(&key), black_box(message))));
        });
    }
    group.finish();

    let mac = hmac::Hmac::<Sha256>::new(&key);
    let mut group = c.benchmark_group("hmac_sha256_reused");
    for size in [32, 1024] {
        let message = vec![0xa5; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", size), &message, |b, message| {
            b.iter(|| black_box(mac.sign(black_box(message))));
        });
    }
    group.finish();

    let message = [0xa5; 1024];
    let mut group = c.benchmark_group("hmac_sha256_key");
    group.throughput(Throughput::Bytes(message.len() as u64));
    for size in [16, 64, 100] {
        let key = vec![0x5a; size];
        group.bench_with_input(BenchmarkId::new("bytes", size), &key, |b, key| {
            b.iter(|| black_box(hmac::hmac::<Sha256>(black_box(key), black_box(&message))));
        });
    }
    group.finish();
}

criterion_group!(benches, hmac_sha256);
criterion_main!(benches);
