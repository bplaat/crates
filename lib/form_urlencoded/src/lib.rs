/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [form_urlencoded](https://crates.io/crates/form_urlencoded) crate

use std::borrow::{Borrow, Cow};
use std::str;

use percent_encoding::{percent_decode, percent_encode_byte};

// MARK: Parse
/// Parse an `application/x-www-form-urlencoded` byte string into an iterator of (name, value) pairs.
pub fn parse(input: &[u8]) -> Parse<'_> {
    Parse { input }
}

/// Iterator over parsed (name, value) pairs.
#[derive(Clone, Copy)]
pub struct Parse<'a> {
    input: &'a [u8],
}

impl<'a> Iterator for Parse<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.input.is_empty() {
                return None;
            }
            let mut parts = self.input.splitn(2, |&b| b == b'&');
            let sequence = parts.next()?;
            self.input = parts.next().unwrap_or(&[]);
            if sequence.is_empty() {
                continue;
            }
            let mut kv = sequence.splitn(2, |&b| b == b'=');
            let name = kv.next()?;
            let value = kv.next().unwrap_or(&[]);
            return Some((decode(name), decode(value)));
        }
    }
}

impl<'a> Parse<'a> {
    /// Return an iterator that yields owned `String` pairs.
    pub fn into_owned(self) -> ParseIntoOwned<'a> {
        ParseIntoOwned { inner: self }
    }
}

/// Like [`Parse`] but yields pairs of `String`.
pub struct ParseIntoOwned<'a> {
    inner: Parse<'a>,
}

impl Iterator for ParseIntoOwned<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
    }
}

fn decode(input: &[u8]) -> Cow<'_, str> {
    // Fast path: no encoding characters — return borrowed slice directly.
    if !input.contains(&b'+') && !input.contains(&b'%') {
        return String::from_utf8_lossy(input);
    }
    let replaced = replace_plus(input);
    let decoded = percent_decode(replaced.as_ref()).into_owned();
    Cow::Owned(
        String::from_utf8(decoded)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
    )
}

fn replace_plus(input: &[u8]) -> Cow<'_, [u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(first) => {
            let mut out = input.to_owned();
            out[first] = b' ';
            for b in &mut out[first + 1..] {
                if *b == b'+' {
                    *b = b' ';
                }
            }
            Cow::Owned(out)
        }
    }
}

// MARK: ByteSerialize
/// Serialize bytes as `application/x-www-form-urlencoded`, yielding `&str` slices.
pub fn byte_serialize(input: &[u8]) -> ByteSerialize<'_> {
    ByteSerialize { bytes: input }
}

/// Iterator returned by [`byte_serialize`].
pub struct ByteSerialize<'a> {
    bytes: &'a [u8],
}

fn is_unchanged(b: u8) -> bool {
    matches!(b, b'*' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z')
}

impl<'a> Iterator for ByteSerialize<'a> {
    type Item = &'a str;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<&'a str> {
        let (&first, tail) = self.bytes.split_first()?;
        if !is_unchanged(first) {
            self.bytes = tail;
            return Some(if first == b' ' {
                "+"
            } else {
                percent_encode_byte(first)
            });
        }
        let end = tail
            .iter()
            .position(|&b| !is_unchanged(b))
            .map_or(self.bytes.len(), |i| i + 1);
        let (chunk, rest) = self.bytes.split_at(end);
        self.bytes = rest;
        // SAFETY: chunk only contains bytes that passed is_unchanged(), which are ASCII alphanumeric and punctuation - all valid UTF-8.
        Some(unsafe { str::from_utf8_unchecked(chunk) })
    }
}

// MARK: Serializer
/// Encoding override function type.
pub type EncodingOverride<'a> = Option<&'a dyn Fn(&str) -> Cow<'_, [u8]>>;

/// Trait for types that can serve as the serialization target.
pub trait Target {
    /// Access the underlying `String`.
    fn as_mut_string(&mut self) -> &mut String;
    /// Consume and return the finished value.
    fn finish(self) -> Self::Finished;
    /// The type returned by [`finish`](Target::finish).
    type Finished;
}

impl Target for String {
    fn as_mut_string(&mut self) -> &mut String {
        self
    }
    fn finish(self) -> String {
        self
    }
    type Finished = String;
}

impl Target for &mut String {
    fn as_mut_string(&mut self) -> &mut String {
        self
    }
    fn finish(self) -> Self {
        self
    }
    type Finished = Self;
}

/// `application/x-www-form-urlencoded` serializer.
pub struct Serializer<'a, T: Target> {
    target: Option<T>,
    start_position: usize,
    encoding: EncodingOverride<'a>,
}

impl<'a, T: Target> Serializer<'a, T> {
    /// Create a new serializer wrapping `target`.
    pub fn new(target: T) -> Self {
        Self::for_suffix(target, 0)
    }

    /// Create a serializer for a suffix of `target` starting at `start_position`.
    pub fn for_suffix(mut target: T, start_position: usize) -> Self {
        assert!(
            target.as_mut_string().len() >= start_position,
            "invalid length {} for target of length {}",
            start_position,
            target.as_mut_string().len()
        );
        Self {
            target: Some(target),
            start_position,
            encoding: None,
        }
    }

    /// Remove all name/value pairs appended so far.
    pub fn clear(&mut self) -> &mut Self {
        let start = self.start_position;
        self.string().truncate(start);
        self
    }

    /// Override the character encoding (always UTF-8 in this minimal impl).
    pub fn encoding_override(&mut self, new: EncodingOverride<'a>) -> &mut Self {
        self.encoding = new;
        self
    }

    /// Append a name/value pair.
    pub fn append_pair(&mut self, name: &str, value: &str) -> &mut Self {
        let start = self.start_position;
        let enc = self.encoding;
        let s = self.string();
        append_pair(s, start, enc, name, value);
        self
    }

    /// Append a name without a value.
    pub fn append_key_only(&mut self, name: &str) -> &mut Self {
        let start = self.start_position;
        let enc = self.encoding;
        let s = self.string();
        append_key_only(s, start, enc, name);
        self
    }

    /// Append multiple name/value pairs.
    pub fn extend_pairs<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let start = self.start_position;
        let enc = self.encoding;
        let s = self.string();
        for pair in iter {
            let (k, v) = pair.borrow();
            append_pair(s, start, enc, k.as_ref(), v.as_ref());
        }
        self
    }

    /// Append multiple keys without values.
    pub fn extend_keys_only<I, K>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Borrow<K>,
        K: AsRef<str>,
    {
        let start = self.start_position;
        let enc = self.encoding;
        let s = self.string();
        for key in iter {
            append_key_only(s, start, enc, key.borrow().as_ref());
        }
        self
    }

    /// Finish serialization and return the target.
    pub fn finish(&mut self) -> T::Finished {
        self.target
            .take()
            .expect("Serializer::finish called twice")
            .finish()
    }

    fn string(&mut self) -> &mut String {
        self.target
            .as_mut()
            .expect("Serializer already finished")
            .as_mut_string()
    }
}

fn append_pair(s: &mut String, start: usize, enc: EncodingOverride<'_>, name: &str, value: &str) {
    append_separator_if_needed(s, start);
    append_encoded(name, s, enc);
    s.push('=');
    append_encoded(value, s, enc);
}

fn append_key_only(s: &mut String, start: usize, enc: EncodingOverride<'_>, name: &str) {
    append_separator_if_needed(s, start);
    append_encoded(name, s, enc);
}

fn append_separator_if_needed(s: &mut String, start: usize) {
    if s.len() > start {
        s.push('&');
    }
}

fn append_encoded(input: &str, s: &mut String, encoding: EncodingOverride<'_>) {
    let bytes: Cow<'_, [u8]> = match encoding {
        Some(f) => f(input),
        None => Cow::Borrowed(input.as_bytes()),
    };
    s.extend(byte_serialize(&bytes));
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let pairs: Vec<_> = parse(b"foo=bar&baz=qux").collect();
        assert_eq!(pairs[0].0, "foo");
        assert_eq!(pairs[0].1, "bar");
        assert_eq!(pairs[1].0, "baz");
        assert_eq!(pairs[1].1, "qux");
    }

    #[test]
    fn test_parse_borrowed() {
        // Plain ASCII with no encoding returns borrowed slices.
        let pairs: Vec<_> = parse(b"key=value").collect();
        assert!(matches!(pairs[0].0, Cow::Borrowed(_)));
        assert!(matches!(pairs[0].1, Cow::Borrowed(_)));
    }

    #[test]
    fn test_parse_percent_decode() {
        let pairs: Vec<_> = parse(b"q=hello%20world").collect();
        assert_eq!(pairs[0].1, "hello world");
    }

    #[test]
    fn test_parse_plus_as_space() {
        let pairs: Vec<_> = parse(b"q=hello+world").collect();
        assert_eq!(pairs[0].1, "hello world");
    }

    #[test]
    fn test_serializer() {
        let encoded = Serializer::new(String::new())
            .append_pair("foo", "bar & baz")
            .append_pair("saison", "\u{00C9}t\u{00E9}+hiver")
            .finish();
        assert_eq!(encoded, "foo=bar+%26+baz&saison=%C3%89t%C3%A9%2Bhiver");
    }

    #[test]
    fn test_byte_serialize() {
        let s: String = byte_serialize(b"a b+c").collect();
        assert_eq!(s, "a+b%2Bc");
    }

    #[test]
    fn test_parse_empty_input() {
        let pairs: Vec<_> = parse(b"").collect();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_parse_key_without_value() {
        let pairs: Vec<_> = parse(b"key").collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "key");
        assert_eq!(pairs[0].1, "");
    }

    #[test]
    fn test_parse_consecutive_separators() {
        // Double && should skip the empty sequence and yield only real pairs
        let pairs: Vec<_> = parse(b"a=1&&b=2").collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "a");
        assert_eq!(pairs[0].1, "1");
        assert_eq!(pairs[1].0, "b");
        assert_eq!(pairs[1].1, "2");
    }

    #[test]
    fn test_parse_into_owned() {
        let pairs: Vec<(String, String)> = parse(b"x=hello+world").into_owned().collect();
        assert_eq!(pairs[0].0, "x");
        assert_eq!(pairs[0].1, "hello world");
    }

    #[test]
    fn test_serializer_clear() {
        let mut s = Serializer::new(String::new());
        s.append_pair("old", "value");
        s.clear();
        s.append_pair("new", "data");
        let result = s.finish();
        assert_eq!(result, "new=data");
    }

    #[test]
    fn test_serializer_extend_pairs() {
        let result = Serializer::new(String::new())
            .extend_pairs([("a", "1"), ("b", "2"), ("c", "3")])
            .finish();
        assert_eq!(result, "a=1&b=2&c=3");
    }

    #[test]
    fn test_serializer_extend_keys_only() {
        let result = Serializer::new(String::new())
            .extend_keys_only::<[&str; 2], &str>(["foo", "bar"])
            .finish();
        assert_eq!(result, "foo&bar");
    }

    #[test]
    fn test_serializer_append_key_only() {
        let result = Serializer::new(String::new())
            .append_pair("x", "1")
            .append_key_only("flag")
            .finish();
        assert_eq!(result, "x=1&flag");
    }

    #[test]
    fn test_byte_serialize_special_chars() {
        // slash → %2F, @ → %40, alphanumerics unchanged
        let slash: String = byte_serialize(b"/").collect();
        assert_eq!(slash, "%2F");
        let at: String = byte_serialize(b"@").collect();
        assert_eq!(at, "%40");
        let alnum: String = byte_serialize(b"abc123").collect();
        assert_eq!(alnum, "abc123");
    }
}
