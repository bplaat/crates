/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

//! A minimal replacement for the [zeroize](https://crates.io/crates/zeroize) crate.

use std::ffi::CString;
use std::marker::{PhantomData, PhantomPinned};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice::IterMut;
use std::sync::atomic::{Ordering, compiler_fence};

/// Securely overwrite a value with zeroes.
pub trait Zeroize {
    /// Overwrite this value using writes that cannot be optimized away.
    fn zeroize(&mut self);
}

/// Marker for types that zeroize themselves when dropped.
pub trait ZeroizeOnDrop {}

/// Marker for types whose default value is their zero representation.
pub trait DefaultIsZeroes: Copy + Default + Sized {}

impl<Z: DefaultIsZeroes> Zeroize for Z {
    fn zeroize(&mut self) {
        // SAFETY: `self` is a valid writable pointer and the value has the same type.
        unsafe { ptr::write_volatile(self, Z::default()) };
        compiler_fence(Ordering::SeqCst);
    }
}

macro_rules! impl_default_is_zeroes {
    ($($type:ty),+ $(,)?) => {
        $(impl DefaultIsZeroes for $type {})+
    };
}

impl_default_is_zeroes!(
    (),
    bool,
    char,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    PhantomPinned,
);

impl<Z: Zeroize, const N: usize> Zeroize for [Z; N] {
    fn zeroize(&mut self) {
        self.iter_mut().zeroize();
    }
}

impl<Z: Zeroize> Zeroize for IterMut<'_, Z> {
    fn zeroize(&mut self) {
        for value in self {
            value.zeroize();
        }
    }
}

impl<Z> Zeroize for [Z]
where
    Z: DefaultIsZeroes,
{
    fn zeroize(&mut self) {
        for value in self {
            // SAFETY: `value` is a valid writable pointer and the value has the same type.
            unsafe { ptr::write_volatile(value, Z::default()) };
        }
        compiler_fence(Ordering::SeqCst);
    }
}

impl<Z> Zeroize for [MaybeUninit<Z>] {
    fn zeroize(&mut self) {
        for value in self {
            // SAFETY: Every bit pattern, including all zeroes, is valid for `MaybeUninit`.
            unsafe { ptr::write_volatile(value, MaybeUninit::zeroed()) };
        }
        compiler_fence(Ordering::SeqCst);
    }
}

impl Zeroize for str {
    fn zeroize(&mut self) {
        // SAFETY: All-zero bytes are valid UTF-8 and the length is unchanged.
        unsafe { self.as_bytes_mut() }.zeroize();
    }
}

impl<Z: Zeroize> Zeroize for Option<Z> {
    fn zeroize(&mut self) {
        if let Some(value) = self.as_mut() {
            value.zeroize();
        }
        *self = None;
        compiler_fence(Ordering::SeqCst);
    }
}

impl<Z> Zeroize for PhantomData<Z> {
    fn zeroize(&mut self) {}
}

impl<Z: Zeroize> Zeroize for Vec<Z> {
    fn zeroize(&mut self) {
        self.iter_mut().zeroize();
        self.clear();
        self.spare_capacity_mut().zeroize();
    }
}

impl<Z: Zeroize> Zeroize for Box<[Z]> {
    fn zeroize(&mut self) {
        self.iter_mut().zeroize();
    }
}

impl Zeroize for Box<str> {
    fn zeroize(&mut self) {
        self.as_mut().zeroize();
    }
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        // SAFETY: The buffer is zeroed and cleared before it can be observed as a `String`.
        unsafe { self.as_mut_vec() }.zeroize();
    }
}

impl Zeroize for CString {
    fn zeroize(&mut self) {
        let mut bytes = std::mem::take(self).into_bytes_with_nul();
        bytes.zeroize();
        *self = CString::default();
    }
}

/// A wrapper that zeroizes its value when dropped.
#[derive(Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct Zeroizing<Z: Zeroize + ?Sized>(Z);

impl<Z: Zeroize> Zeroizing<Z> {
    /// Wrap a value so it is zeroized when dropped.
    pub const fn new(value: Z) -> Self {
        Self(value)
    }
}

impl<Z: Zeroize + Clone> Clone for Zeroizing<Z> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.zeroize();
        self.0.clone_from(&source.0);
    }
}

impl<Z: Zeroize> From<Z> for Zeroizing<Z> {
    fn from(value: Z) -> Self {
        Self(value)
    }
}

impl<Z: Zeroize + ?Sized> Deref for Zeroizing<Z> {
    type Target = Z;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Z: Zeroize + ?Sized> DerefMut for Zeroizing<Z> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ?Sized, Z> AsRef<T> for Zeroizing<Z>
where
    Z: AsRef<T> + Zeroize + ?Sized,
{
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: ?Sized, Z> AsMut<T> for Zeroizing<Z>
where
    Z: AsMut<T> + Zeroize + ?Sized,
{
    fn as_mut(&mut self) -> &mut T {
        self.0.as_mut()
    }
}

impl<Z: Zeroize + ?Sized> Zeroize for Zeroizing<Z> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<Z: Zeroize + ?Sized> ZeroizeOnDrop for Zeroizing<Z> {}

impl<Z: Zeroize + ?Sized> Drop for Zeroizing<Z> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(feature = "serde")]
impl<Z> serde::Serialize for Zeroizing<Z>
where
    Z: Zeroize + serde::Serialize + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, Z> serde::Deserialize<'de> for Zeroizing<Z>
where
    Z: Zeroize + serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Z::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroizes_slice() {
        let mut secret = [1_u8, 2, 3, 4];
        secret.zeroize();
        assert_eq!(secret, [0; 4]);
    }

    #[test]
    fn zeroizes_vector_and_capacity() {
        let mut secret = Vec::with_capacity(16);
        secret.extend_from_slice(b"password");
        secret.zeroize();
        assert!(secret.is_empty());
        assert!(secret.spare_capacity_mut().iter().all(|byte| {
            // SAFETY: `zeroize` initialized every spare-capacity byte.
            unsafe { byte.assume_init() == 0 }
        }));
    }

    #[test]
    fn zeroizing_dereferences_to_inner_value() {
        let mut secret = Zeroizing::new(String::from("secret"));
        secret.push('!');
        assert_eq!(secret.as_str(), "secret!");
    }
}
