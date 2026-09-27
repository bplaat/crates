/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn decode(c: &mut Criterion) {
    let mut raster = c.benchmark_group("image_decode_raster");
    for (name, enabled, bytes) in [
        (
            "qoi",
            cfg!(feature = "qoi"),
            &include_bytes!("../tests/images/qoi/testcard_rgba.qoi")[..],
        ),
        (
            "jpeg",
            cfg!(feature = "jpeg"),
            &include_bytes!("../tests/images/jpeg/progressive/test.jpg")[..],
        ),
        (
            "png",
            cfg!(feature = "png"),
            &include_bytes!("../tests/images/png/16bpc/basn6a16.png")[..],
        ),
        (
            "gif",
            cfg!(feature = "gif"),
            &include_bytes!("../tests/images/gif/anim/mixed-disposal.gif")[..],
        ),
        (
            "bmp",
            cfg!(feature = "bmp"),
            &include_bytes!("../tests/images/bmp/Info_A8_R8_G8_B8.bmp")[..],
        ),
        (
            "ico",
            cfg!(feature = "ico"),
            &include_bytes!("../tests/images/ico/png-32bpp-alpha.ico")[..],
        ),
    ] {
        if !enabled {
            continue;
        }
        raster.throughput(Throughput::Bytes(bytes.len() as u64));
        raster.bench_function(name, |b| {
            b.iter(|| black_box(image::decode(black_box(bytes)).expect("valid raster image")));
        });
    }
    raster.finish();

    let mut vector = c.benchmark_group("image_decode_vector");
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
        vector.throughput(Throughput::Bytes(bytes.len() as u64));
        vector.bench_function(name, |b| {
            b.iter(|| {
                black_box(image::decode_vector(black_box(bytes)).expect("valid vector image"))
            });
        });
    }
    vector.finish();
}

criterion_group!(benches, decode);
criterion_main!(benches);
