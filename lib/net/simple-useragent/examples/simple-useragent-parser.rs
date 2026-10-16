/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! An example of the user agent parser

use simple_useragent::UserAgentParser;

fn main() {
    // Create a user agent parser
    let parser = UserAgentParser::new();

    // Parse a user agent string
    let ua = parser.parse(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0",
    );

    // Print the parsed user agent fields
    println!("Client family: {}", ua.client.family);
    println!("Client version: {:?}", ua.client.version);
    println!("OS family: {}", ua.os.family);
    println!("OS version: {:?}", ua.os.version);
}
