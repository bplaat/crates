/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void};

/// Encoding types for Objective-C
#[allow(missing_docs)]
pub enum Encoding {
    Char,
    Short,
    Int,
    Long,
    LongLong,
    UChar,
    UShort,
    UInt,
    ULong,
    ULongLong,
    Float,
    Double,
    LongDouble,
    FloatComplex,
    DoubleComplex,
    LongDoubleComplex,
    Bool,
    Void,
    String,
    Object,
    Block,
    Class,
    Sel,
    Unknown,
    BitField(u8, Option<&'static (u64, Encoding)>),
    Pointer(&'static Encoding),
    Atomic(&'static Encoding),
    Array(u64, &'static Encoding),
    Struct(&'static str, &'static [Encoding]),
    Union(&'static str, &'static [Encoding]),
    None,
}
impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char => write!(f, "c"),
            Self::Short => write!(f, "s"),
            Self::Int => write!(f, "i"),
            Self::Long => write!(f, "l"),
            Self::LongLong => write!(f, "q"),
            Self::UChar => write!(f, "C"),
            Self::UShort => write!(f, "S"),
            Self::UInt => write!(f, "I"),
            Self::ULong => write!(f, "L"),
            Self::ULongLong => write!(f, "Q"),
            Self::Float => write!(f, "f"),
            Self::Double => write!(f, "d"),
            Self::LongDouble => write!(f, "D"),
            Self::FloatComplex => write!(f, "jf"),
            Self::DoubleComplex => write!(f, "jd"),
            Self::LongDoubleComplex => write!(f, "jD"),
            Self::Bool => write!(f, "B"),
            Self::Void => write!(f, "v"),
            Self::String => write!(f, "*"),
            Self::Object => write!(f, "@"),
            Self::Block => write!(f, "@?"),
            Self::Class => write!(f, "#"),
            Self::Sel => write!(f, ":"),
            Self::Unknown => write!(f, "?"),
            Self::BitField(size, _) => write!(f, "b{size}"),
            Self::Pointer(ty) => write!(f, "^{ty}"),
            Self::Atomic(ty) => write!(f, "A{ty}"),
            Self::Array(len, ty) => write!(f, "[{len}{ty}]"),
            Self::Struct(name, fields) => {
                write!(f, "{{{name}=")?;
                for field in fields.iter() {
                    write!(f, "{field}")?;
                }
                write!(f, "}}")
            }
            Self::Union(name, fields) => {
                write!(f, "({name}=")?;
                for field in fields.iter() {
                    write!(f, "{field}")?;
                }
                write!(f, ")")
            }
            Self::None => Ok(()),
        }
    }
}

/// Trait for types that can be encoded in Objective-C
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Encode {
    /// The encoding of the type
    const ENCODING: Encoding;
}

// Implementations for primitive types
unsafe impl Encode for () {
    const ENCODING: Encoding = Encoding::Void;
}
unsafe impl Encode for i8 {
    const ENCODING: Encoding = Encoding::Char;
}
unsafe impl Encode for i16 {
    const ENCODING: Encoding = Encoding::Short;
}
unsafe impl Encode for i32 {
    const ENCODING: Encoding = Encoding::Int;
}
unsafe impl Encode for i64 {
    const ENCODING: Encoding = Encoding::LongLong;
}
unsafe impl Encode for u8 {
    const ENCODING: Encoding = Encoding::UChar;
}
unsafe impl Encode for u16 {
    const ENCODING: Encoding = Encoding::UShort;
}
unsafe impl Encode for u32 {
    const ENCODING: Encoding = Encoding::UInt;
}
unsafe impl Encode for u64 {
    const ENCODING: Encoding = Encoding::ULongLong;
}
unsafe impl Encode for f32 {
    const ENCODING: Encoding = Encoding::Float;
}
unsafe impl Encode for f64 {
    const ENCODING: Encoding = Encoding::Double;
}
unsafe impl Encode for bool {
    const ENCODING: Encoding = Encoding::Bool;
}
unsafe impl Encode for isize {
    #[cfg(target_pointer_width = "64")]
    const ENCODING: Encoding = Encoding::LongLong;
    #[cfg(target_pointer_width = "32")]
    const ENCODING: Encoding = Encoding::Long;
}
unsafe impl Encode for usize {
    #[cfg(target_pointer_width = "64")]
    const ENCODING: Encoding = Encoding::ULongLong;
    #[cfg(target_pointer_width = "32")]
    const ENCODING: Encoding = Encoding::UInt;
}
unsafe impl Encode for *const c_void {
    const ENCODING: Encoding = Encoding::Pointer(&Encoding::Void);
}
unsafe impl Encode for *mut c_void {
    const ENCODING: Encoding = Encoding::Pointer(&Encoding::Void);
}
unsafe impl Encode for *const c_char {
    const ENCODING: Encoding = Encoding::String;
}
unsafe impl Encode for *mut c_char {
    const ENCODING: Encoding = Encoding::String;
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::ffi::{c_char, c_void};

    use super::*;
    use crate::runtime::Bool;

    #[test]
    fn test_encoding_display() {
        assert_eq!(Encoding::Void.to_string(), "v");
        assert_eq!(Encoding::Object.to_string(), "@");
        assert_eq!(Encoding::Sel.to_string(), ":");
        assert_eq!(Encoding::Bool.to_string(), "B");
        assert_eq!(Encoding::Int.to_string(), "i");
        assert_eq!(Encoding::UInt.to_string(), "I");
        assert_eq!(Encoding::LongLong.to_string(), "q");
        assert_eq!(Encoding::ULongLong.to_string(), "Q");
        assert_eq!(Encoding::Float.to_string(), "f");
        assert_eq!(Encoding::Double.to_string(), "d");
        assert_eq!(Encoding::String.to_string(), "*");
        assert_eq!(Encoding::Block.to_string(), "@?");
        assert_eq!(Encoding::Pointer(&Encoding::Void).to_string(), "^v");
        assert_eq!(
            Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]).to_string(),
            "{CGPoint=dd}"
        );
    }

    #[test]
    fn test_encode_impls() {
        assert_eq!(<()>::ENCODING.to_string(), "v");
        assert_eq!(i32::ENCODING.to_string(), "i");
        assert_eq!(u32::ENCODING.to_string(), "I");
        assert_eq!(i64::ENCODING.to_string(), "q");
        assert_eq!(u64::ENCODING.to_string(), "Q");
        assert_eq!(f64::ENCODING.to_string(), "d");
        assert_eq!(bool::ENCODING.to_string(), "B");
        assert_eq!(usize::ENCODING.to_string(), "Q");
        assert_eq!(isize::ENCODING.to_string(), "q");
        assert_eq!(<*const c_void>::ENCODING.to_string(), "^v");
        assert_eq!(<*mut c_void>::ENCODING.to_string(), "^v");
        assert_eq!(<*const c_char>::ENCODING.to_string(), "*");
        assert_eq!(Bool::ENCODING.to_string(), "B");
    }
}
