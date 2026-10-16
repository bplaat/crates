/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("gzip_decompress");
    for (name, data, size) in [
        (
            "small",
            &include_bytes!("../tests/fixtures/small.gz")[..],
            11,
        ),
        (
            "large",
            &include_bytes!("../tests/fixtures/large.gz")[..],
            1024 * 1024,
        ),
    ] {
        assert_eq!(
            gzip::decompress(data).expect("valid gzip member").len(),
            size
        );
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", name), &data, |b, data| {
            b.iter(|| black_box(gzip::decompress(black_box(data)).expect("valid gzip member")));
        });
    }
    group.finish();
}

fn compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("gzip_compress");
    for (name, data) in [
        ("small", &include_bytes!("../tests/fixtures/small.gz")[..]),
        ("large", &include_bytes!("../tests/fixtures/large.gz")[..]),
    ] {
        let input = gzip::decompress(data).expect("valid gzip member");
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("bytes", name), &input, |b, input| {
            b.iter(|| black_box(gzip::compress(black_box(input))));
        });
    }
    group.finish();
}

criterion_group!(benches, decompress, compress);
criterion_main!(benches);
