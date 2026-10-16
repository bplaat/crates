/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Parser construction and common user-agent parsing paths.

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use simple_useragent::UserAgentParser;

fn user_agent(c: &mut Criterion) {
    let mut creation = c.benchmark_group("useragent_create");
    creation.bench_function("parser", |b| {
        b.iter(|| black_box(UserAgentParser::new()));
    });
    creation.finish();

    let parser = UserAgentParser::new();
    let mut parsing = c.benchmark_group("useragent_parse");
    for (name, input) in [
        (
            "firefox",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0",
        ),
        (
            "chrome",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        ),
        ("unknown", "UnknownUserAgent/1.0"),
    ] {
        parsing.bench_function(name, |b| {
            b.iter(|| black_box(parser.parse(black_box(input))));
        });
    }
    parsing.finish();
}

criterion_group!(benches, user_agent);
criterion_main!(benches);
