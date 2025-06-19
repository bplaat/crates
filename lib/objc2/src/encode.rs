/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;

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
            Self::BitField(size, _) => write!(f, "b{}", size),
            Self::Pointer(ty) => write!(f, "^{}", ty),
            Self::Atomic(ty) => write!(f, "A{}", ty),
            Self::Array(len, ty) => write!(f, "[{len}{ty}]"),
            Self::Struct(name, fields) => {
                write!(f, "{{{}", name)?;
                for field in fields.iter() {
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            Self::Union(name, fields) => {
                write!(f, "({}", name)?;
                for field in fields.iter() {
                    write!(f, "{}", field)?;
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
unsafe impl Encode for *const c_void {
    const ENCODING: Encoding = Encoding::Pointer(&Encoding::Void);
}
unsafe impl Encode for *mut c_void {
    const ENCODING: Encoding = Encoding::Pointer(&Encoding::Void);
}
