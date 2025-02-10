/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{RawStatement, Value};

/// A trait for binding values to a statement
pub trait Bind {
    /// Bind values to a statement
    fn bind(self, statement: &mut RawStatement);
}

impl Bind for () {
    fn bind(self, _statement: &mut RawStatement) {}
}

impl<T> Bind for T
where
    T: Into<Value>,
{
    fn bind(self, statement: &mut RawStatement) {
        statement.bind_value(0, self.into());
    }
}

macro_rules! impl_bind_for_tuple {
    ($($n:tt: $t:ident),*) => (
        impl<$($t,)*> Bind for ($($t,)*)
        where
            $($t: Into<Value>,)+
        {
            fn bind(self, statement: &mut RawStatement)  {
                $( statement.bind_value($n, self.$n.into()); )*
            }
        }
    );
}
impl_bind_for_tuple!(0: A);
impl_bind_for_tuple!(0: A, 1: B);
impl_bind_for_tuple!(0: A, 1: B, 2: C);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N, 14: O);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M, 13: N, 14: O, 15: P);
