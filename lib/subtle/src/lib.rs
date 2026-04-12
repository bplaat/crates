/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

//! A minimal replacement for the [subtle](https://crates.io/crates/subtle) crate

use std::ops::Not;
use std::ptr;

// MARK: Choice
/// A boolean value whose representation is secret.
#[derive(Clone, Copy, Debug)]
pub struct Choice(u8);

impl Choice {
    /// Unwrap the `Choice` as a `u8` (0 or 1).
    pub fn unwrap_u8(self) -> u8 {
        self.0
    }
}

impl From<Choice> for bool {
    fn from(c: Choice) -> bool {
        c.0 != 0
    }
}

impl Not for Choice {
    type Output = Choice;
    fn not(self) -> Choice {
        Choice(1 - self.0)
    }
}

// MARK: ConstantTimeEq
/// A trait for constant-time equality comparisons.
pub trait ConstantTimeEq {
    /// Determine if two items are equal in constant time, returning a [`Choice`].
    fn ct_eq(&self, other: &Self) -> Choice;

    /// Determine if two items are unequal in constant time, returning a [`Choice`].
    fn ct_ne(&self, other: &Self) -> Choice {
        !self.ct_eq(other)
    }
}

impl ConstantTimeEq for [u8] {
    fn ct_eq(&self, other: &[u8]) -> Choice {
        if self.len() != other.len() {
            return Choice(0);
        }
        let mut diff = 0u8;
        for (x, y) in self.iter().zip(other.iter()) {
            // SAFETY: Both `x` and `y` are valid references from our own slices.
            // Volatile reads prevent the compiler from eliminating or short-circuiting
            // the loop, which is required for constant-time behavior.
            diff |= unsafe { ptr::read_volatile(x) ^ ptr::read_volatile(y) };
        }
        // Map diff to Choice: diff==0 (equal) -> Choice(1), diff!=0 -> Choice(0).
        // `diff | diff.wrapping_neg()` sets the high bit iff diff != 0.
        Choice(1 - ((diff | diff.wrapping_neg()) >> 7))
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_equal_slices() {
        assert!(bool::from(b"hello".ct_eq(b"hello")));
    }

    #[test]
    fn test_unequal_slices() {
        assert!(!bool::from(b"hello".ct_eq(b"world")));
    }

    #[test]
    fn test_different_lengths() {
        assert!(!bool::from(b"hello".ct_eq(b"hello!")));
    }

    #[test]
    fn test_empty_slices() {
        assert!(bool::from(b"".ct_eq(b"")));
    }

    #[test]
    fn test_ct_ne() {
        assert!(bool::from(b"hello".ct_ne(b"world")));
        assert!(!bool::from(b"hello".ct_ne(b"hello")));
    }

    #[test]
    fn test_choice_not() {
        assert!(!bool::from(!Choice(1)));
        assert!(bool::from(!Choice(0)));
    }
}
