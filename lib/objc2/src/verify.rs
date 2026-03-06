/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// Debug-build encoding verification for `msg_send!`.

use std::ffi::CStr;

use crate::encode::Encoding;
use crate::ffi::{class_getInstanceMethod, class_getName, method_getTypeEncoding, object_getClass};
use crate::runtime::{AnyObject, Sel};

/// Consume one complete ObjC type token from `s`, skipping trailing digits (offset).
/// Returns `(token, remaining)` or `None` if empty.
/// Returns the byte length of one ObjC type token in `s` (no leading/trailing digit stripping).
fn enc_type_len(s: &str) -> Option<usize> {
    let first = s.chars().next()?;
    match first {
        '{' => matching_close(s, '{', '}'),
        '(' => matching_close(s, '(', ')'),
        '[' => matching_close(s, '[', ']'),
        '^' => enc_type_len(&s[1..]).map(|n| 1 + n),
        '@' if s.starts_with("@?") => Some(2),
        'r' | 'n' | 'N' | 'o' | 'O' | 'R' | 'V' => enc_type_len(&s[1..]).map(|n| 1 + n),
        c => Some(c.len_utf8()),
    }
}

fn next_enc_type(s: &str) -> Option<(&str, &str)> {
    // Strip leading digits (frame size / stack offset from previous token)
    let s = s.trim_start_matches(|c: char| c.is_ascii_digit() || c == '-');
    if s.is_empty() {
        return None;
    }
    let token_end = enc_type_len(s)?;
    let (token, rest) = s.split_at(token_end);
    // Skip trailing digits (the stack offset after this token)
    let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
    Some((token, rest))
}

fn matching_close(s: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(i + c.len_utf8());
            }
        }
    }
    None
}

fn class_name(cls: *mut std::ffi::c_void) -> String {
    unsafe {
        CStr::from_ptr(class_getName(cls))
            .to_string_lossy()
            .into_owned()
    }
}

fn sel_name(sel: Sel) -> String {
    if sel.0.is_null() {
        return String::from("<unknown>");
    }
    unsafe {
        CStr::from_ptr(crate::ffi::sel_getName(sel.0))
            .to_string_lossy()
            .into_owned()
    }
}

/// Strip leading ObjC type modifier chars (`r`=const, `n`/`N`/`o`/`O`/`R`/`V` = in/out/inout/bycopy/byref/oneway).
fn strip_modifiers(s: &str) -> &str {
    s.trim_start_matches(['r', 'n', 'N', 'o', 'O', 'R', 'V'])
}

/// Lenient encoding comparison:
/// - Leading ObjC type modifiers (`r`=const, etc.) are stripped before comparison.
/// - Exact match (after stripping) always passes.
/// - `^v` matches any pointer (`^...`) and vice versa.
/// - Signed/unsigned variants of same-width integers are interchangeable.
/// - Structs/unions are compared by name only.
fn enc_match(actual: &str, expected: &str) -> bool {
    let actual = strip_modifiers(actual);
    let expected = strip_modifiers(expected);
    if actual == expected {
        return true;
    }
    // Relax block vs object: @? (block) and @ (object) are both pointer-sized; null is valid for either
    if (actual == "@?" || actual == "@") && (expected == "@?" || expected == "@") {
        return true;
    }
    // Relax void-pointer: ^v matches any ^T and vice versa
    if (actual == "^v" || expected == "^v") && actual.starts_with('^') && expected.starts_with('^')
    {
        return true;
    }
    // Relax sign for integer types
    fn sign_relax(s: &str) -> &str {
        match s {
            "c" | "C" => "c",
            "s" | "S" => "s",
            "i" | "I" => "i",
            "l" | "L" => "l",
            "q" | "Q" => "q",
            other => other,
        }
    }
    if sign_relax(actual) == sign_relax(expected) {
        return true;
    }
    // Relax structs: compare name part only
    if actual.starts_with('{') && expected.starts_with('{') {
        let a_name = actual[1..]
            .split('=')
            .next()
            .unwrap_or("")
            .trim_end_matches('}');
        let e_name = expected[1..]
            .split('=')
            .next()
            .unwrap_or("")
            .trim_end_matches('}');
        return a_name == e_name;
    }
    false
}

/// Verify that sending `sel` to `obj` with Rust types `args`/`ret` matches the ObjC method
/// signature. Panics with a descriptive message on any mismatch.
/// Called by `msg_send!` in debug builds only; zero overhead in release.
pub(crate) fn verify_send(obj: *mut AnyObject, sel: Sel, args: &[Encoding], ret: &Encoding) {
    unsafe {
        let cls = object_getClass(obj as *const AnyObject);
        if cls.is_null() {
            return;
        }

        let method = class_getInstanceMethod(cls, sel.0);
        if method.is_null() {
            panic!(
                "invalid message send to -[{} {}]: method not found",
                class_name(cls),
                sel_name(sel),
            );
        }

        let enc_ptr = method_getTypeEncoding(method);
        if enc_ptr.is_null() {
            return;
        }
        let enc_str = CStr::from_ptr(enc_ptr).to_string_lossy();
        let enc = enc_str.as_ref();

        // First token is return type
        let Some((actual_ret, mut rest)) = next_enc_type(enc) else {
            return;
        };

        let expected_ret = ret.to_string();
        if !enc_match(actual_ret, &expected_ret) {
            panic!(
                "invalid message send to -[{} {}]: expected return type '{}' but found '{}'",
                class_name(cls),
                sel_name(sel),
                expected_ret,
                actual_ret,
            );
        }

        // Skip implicit receiver (`@`) and selector (`:`) args
        for _ in 0..2 {
            if let Some((_, r)) = next_enc_type(rest) {
                rest = r;
            } else {
                return;
            }
        }

        // Verify each explicit argument
        for (i, arg_enc) in args.iter().enumerate() {
            match next_enc_type(rest) {
                None => {
                    panic!(
                        "invalid message send to -[{} {}]: too many arguments (method has {}, got {})",
                        class_name(cls),
                        sel_name(sel),
                        i,
                        args.len(),
                    );
                }
                Some((actual_arg, r)) => {
                    rest = r;
                    let expected_arg = arg_enc.to_string();
                    if !enc_match(actual_arg, &expected_arg) {
                        panic!(
                            "invalid message send to -[{} {}]: argument {} expected '{}' but found '{}'",
                            class_name(cls),
                            sel_name(sel),
                            i,
                            expected_arg,
                            actual_arg,
                        );
                    }
                }
            }
        }

        // Ensure method doesn't expect more arguments
        if next_enc_type(rest).is_some() {
            panic!(
                "invalid message send to -[{} {}]: too few arguments (got {}, method expects more)",
                class_name(cls),
                sel_name(sel),
                args.len(),
            );
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use crate::runtime::AnyObject;
    use crate::{class, msg_send};

    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {}

    #[test]
    #[should_panic(expected = "invalid message send")]
    fn test_verify_wrong_return_type() {
        // NSString's -length returns NSUInteger (Q), not a pointer object (@)
        let ns: *mut AnyObject = unsafe { msg_send![class!(NSString), alloc] };
        let ns: *mut AnyObject = unsafe { msg_send![ns, init] };
        // Intentionally use the wrong return type (pointer instead of integer)
        let _wrong: *mut AnyObject = unsafe { msg_send![ns, length] };
    }
}
