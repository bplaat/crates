/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple small-http client pool example

use small_http::{Client, Request};

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0";

fn main() {
    let mut client = Client::new().header("User-Agent", USER_AGENT);
    for i in 0..10 {
        let res = client
            .fetch(Request::get("http://ipinfo.io/json"))
            .expect("Can't fetch");
        println!("{}: {}", i, String::from_utf8_lossy(&res.body));
    }
}
