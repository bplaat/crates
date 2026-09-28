/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

// Builds a bottom-up 24-bit BMP gradient, avoiding a large checked-in fixture.
fn bmp(width: u32, height: u32) -> Vec<u8> {
    let stride = (width * 3).next_multiple_of(4);
    let offset = 54u32;
    let size = offset + stride * height;
    let mut out = Vec::with_capacity(size as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&offset.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    for y in 0..height {
        for x in 0..width {
            out.extend_from_slice(&[x as u8, y as u8, (x ^ y) as u8]);
        }
        out.resize(out.len() + (stride - width * 3) as usize, 0);
    }
    out
}

fn decode(c: &mut Criterion) {
    let bmp = bmp(800, 600);
    let mut raster = c.benchmark_group("image_decode_raster");
    for (name, enabled, bytes) in [
        (
            "qoi",
            cfg!(feature = "qoi"),
            &include_bytes!("images/dice.qoi")[..],
        ),
        (
            "jpeg_baseline",
            cfg!(feature = "jpeg"),
            &include_bytes!("images/dice-baseline.jpg")[..],
        ),
        (
            "jpeg_progressive",
            cfg!(feature = "jpeg"),
            &include_bytes!("images/dice-progressive.jpg")[..],
        ),
        (
            "jpeg_restart_420",
            cfg!(feature = "jpeg"),
            &include_bytes!("../tests/images/jpeg/subsampled/restart-420.jpg")[..],
        ),
        (
            "png",
            cfg!(feature = "png"),
            &include_bytes!("images/dice.png")[..],
        ),
        (
            "png_interlaced",
            cfg!(feature = "png"),
            &include_bytes!("../tests/images/png/interlaced/basi2c08.png")[..],
        ),
        (
            "png_rgba16",
            cfg!(feature = "png"),
            &include_bytes!("../tests/images/png/16bpc/basn6a16.png")[..],
        ),
        (
            "apng",
            cfg!(feature = "png"),
            &include_bytes!("../tests/images/png/apng/wpt/fcTL-dispose-in-region-previous.png")[..],
        ),
        (
            "gif",
            cfg!(feature = "gif"),
            &include_bytes!("images/dice.gif")[..],
        ),
        (
            "gif_animated",
            cfg!(feature = "gif"),
            &include_bytes!("images/dice-animated.gif")[..],
        ),
        (
            "gif_interlaced",
            cfg!(feature = "gif"),
            &include_bytes!("../tests/images/gif/anim/interlaced.gif")[..],
        ),
        ("bmp", cfg!(feature = "bmp"), &bmp[..]),
        (
            "ico",
            cfg!(feature = "ico"),
            &include_bytes!("../tests/images/ico/png-32bpp-alpha.ico")[..],
        ),
    ] {
        if !enabled {
            continue;
        }
        let decoded = image::decode(bytes).expect("valid raster image");
        let pixels = u64::from(decoded.width())
            * u64::from(decoded.height())
            * decoded.frames().len() as u64;
        raster.throughput(Throughput::Elements(pixels));
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
