/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Database initialization, IP lookup, and typed record decoding.

#![allow(missing_docs)]

use std::hint::black_box;
use std::net::IpAddr;

use criterion::{Criterion, criterion_group, criterion_main};
use maxminddb::{Reader, geoip2};

const DATABASE: &[u8] = include_bytes!("../test-data/GeoLite2-City-Test.mmdb");

fn database(c: &mut Criterion) {
    let mut open = c.benchmark_group("maxminddb_open");
    open.bench_function("from_source", |b| {
        b.iter(|| black_box(Reader::from_source(black_box(DATABASE)).expect("valid database")));
    });
    open.finish();

    let reader = Reader::from_source(DATABASE).expect("valid database");
    let hit: IpAddr = "89.160.20.128".parse().expect("valid IP address");
    let miss: IpAddr = "192.0.2.1".parse().expect("valid IP address");
    assert!(reader.lookup(hit).expect("valid lookup").has_data());
    assert!(!reader.lookup(miss).expect("valid lookup").has_data());

    let mut lookup = c.benchmark_group("maxminddb_lookup");
    for (name, address) in [("ipv4_hit", hit), ("ipv4_miss", miss)] {
        lookup.bench_function(name, |b| {
            b.iter(|| black_box(reader.lookup(black_box(address)).expect("valid lookup")));
        });
    }
    lookup.finish();

    let result = reader.lookup(hit).expect("valid lookup");
    let mut decode = c.benchmark_group("maxminddb_decode");
    decode.bench_function("city", |b| {
        b.iter(|| black_box(result.decode::<geoip2::City>().expect("valid city record")));
    });
    decode.finish();
}

criterion_group!(benches, database);
criterion_main!(benches);
