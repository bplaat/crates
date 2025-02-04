/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! An example of the user agent parser

use simple_useragent::UserAgentParser;

fn main() {
    let parser = UserAgentParser::new();
    let ua = parser.parse(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0",
    );
    println!("Client Family: {}", ua.client.family);
    println!("Client Version: {:?}", ua.client.version);
    println!("OS Family: {}", ua.os.family);
    println!("OS Version: {:?}", ua.os.version);
}
