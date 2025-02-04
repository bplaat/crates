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
    assert_eq!(ua.client.family, "Firefox");
    assert_eq!(ua.client.version, Some("134.0".to_string()));
    assert_eq!(ua.os.family, "Mac OS X");
    assert_eq!(ua.os.version, Some("10.15".to_string()));
}
