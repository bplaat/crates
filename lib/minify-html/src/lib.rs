/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple HTML minifier library

use std::fs;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_COMMENTS: Regex = Regex::new(r"<!--.*?-->").expect("Should compile");
    static ref RE_WHITESPACE_BETWEEN_TAGS: Regex = Regex::new(r">\s+<").expect("Should compile");
    static ref RE_LEADING_TRAILING_WHITESPACE: Regex =
        Regex::new(r"^\s+|\s+$").expect("Should compile");
    static ref RE_MULTIPLE_SPACES: Regex = Regex::new(r"\s{2,}").expect("Should compile");
}

/// Minify the given html
pub fn minify(html: impl AsRef<str>) -> String {
    fn inner(html: &str) -> String {
        let mut result = html.to_string();
        result = RE_COMMENTS.replace_all(&result, "").to_string();
        result = RE_WHITESPACE_BETWEEN_TAGS
            .replace_all(&result, "><")
            .to_string();
        result = RE_LEADING_TRAILING_WHITESPACE
            .replace_all(&result, "")
            .to_string();
        result = RE_MULTIPLE_SPACES.replace_all(&result, " ").to_string();
        result
    }
    inner(html.as_ref())
}

/// Minify the html file at the given input path and write the minified html to the output path
pub fn minify_file(
    input_path: impl AsRef<str>,
    output_path: impl AsRef<str>,
) -> std::io::Result<()> {
    let html = fs::read_to_string(input_path.as_ref())?;
    fs::write(output_path.as_ref(), minify(&html))?;
    Ok(())
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_minify_removes_comments() {
        let html = "<!-- This is a comment --><p>Hello</p>";
        let expected = "<p>Hello</p>";
        assert_eq!(minify(html), expected);
    }

    #[test]
    fn test_minify_removes_whitespace_between_tags() {
        let html = "<p>Hello</p>   <p>World</p>";
        let expected = "<p>Hello</p><p>World</p>";
        assert_eq!(minify(html), expected);
    }

    #[test]
    fn test_minify_removes_leading_trailing_whitespace() {
        let html = "   <p>Hello</p>   ";
        let expected = "<p>Hello</p>";
        assert_eq!(minify(html), expected);
    }

    #[test]
    fn test_minify_reduces_multiple_spaces() {
        let html = "<p>Hello   World</p>";
        let expected = "<p>Hello World</p>";
        assert_eq!(minify(html), expected);
    }

    #[test]
    fn test_minify_combined() {
        let html = "   <!-- Comment --><p>  Hello   World  </p>   ";
        let expected = "<p> Hello World </p>";
        assert_eq!(minify(html), expected);
    }
}
