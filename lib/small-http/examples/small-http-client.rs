/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple small-http client example

use serde::Deserialize;
use small_http::Request;

#[derive(Deserialize)]
struct IpInfo {
    hostname: String,
}

fn main() {
    let res = Request::get("http://ipinfo.io/json")
        .fetch()
        .expect("Can't fetch");
    println!("{}", String::from_utf8_lossy(&res.body));
    let ip_info = res.into_json::<IpInfo>().expect("Can't parse JSON");
    println!("Hostname: {}", ip_info.hostname);
}
