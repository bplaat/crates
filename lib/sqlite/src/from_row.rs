/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{RawStatement, Value};

/// A trait for converting read values from a statement to a row
pub trait FromRow: Sized {
    /// Convert read values from a statement to a row
    fn from_row(statement: &mut RawStatement) -> Self;
}

impl FromRow for () {
    fn from_row(_statement: &mut RawStatement) -> Self {}
}

impl<T> FromRow for T
where
    T: TryFrom<Value>,
{
    fn from_row(statement: &mut RawStatement) -> Self {
        match T::try_from(statement.read_value(0)) {
            Ok(value) => value,
            Err(_) => panic!("Can't convert Value"),
        }
    }
}

macro_rules! impl_from_row_for_tuple {
    ($($n:tt: $t:ident),*) => (
        impl<$($t,)*> FromRow for ($($t,)*)
        where
            $($t: TryFrom<Value>,)+
        {
            fn from_row(statement: &mut RawStatement) -> Self {
                (
                    $(
                        match $t::try_from(statement.read_value($n)) {
                            Ok(value) => value,
                            Err(_) => panic!("Can't convert Value"),
                        },
                    )*
                )
            }
        }
    );
}
impl_from_row_for_tuple!(0: A);
impl_from_row_for_tuple!(0: A, 1: B);
impl_from_row_for_tuple!(0: A, 1: B, 2: C);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N, 14: O);
impl_from_row_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N, 14: O, 15: P);
