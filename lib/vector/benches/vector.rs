/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_decode");
    for (name, enabled, bytes) in [
        (
            "svg",
            cfg!(feature = "svg"),
            &include_bytes!("../tests/images/svg/painting/reftests/paint-order-002.svg")[..],
        ),
        (
            "tinyvg",
            cfg!(feature = "tinyvg"),
            &include_bytes!("../tests/images/tinyvg/flowchart.tvg")[..],
        ),
    ] {
        if !enabled {
            continue;
        }
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(vector::decode_vector(black_box(bytes)).expect("valid vector image"))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, decode);
criterion_main!(benches);
