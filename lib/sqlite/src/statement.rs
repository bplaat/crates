/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::marker::PhantomData;

use crate::error::{Error, Result};
use crate::sys::*;
use crate::value::Value;

// MARK: Bind
pub trait Bind {
    fn bind(self, statement: *mut sqlite3_stmt) -> Result<()>;
}

impl Bind for () {
    fn bind(self, _statement: *mut sqlite3_stmt) -> Result<()> {
        Ok(())
    }
}

impl<T> Bind for T
where
    T: Into<Value>,
{
    fn bind(self, statement: *mut sqlite3_stmt) -> Result<()> {
        self.into().bind_to_statement(statement, 1)
    }
}

macro_rules! impl_bind_for_tuple {
    ($($n:tt: $t:ident),*) => (
        impl<$($t,)*> Bind for ($($t,)*)
        where
            $($t: Into<Value>,)+
        {
            fn bind(self, statement: *mut sqlite3_stmt) -> Result<()> {
                $( self.$n.into().bind_to_statement(statement, $n + 1)?; )*
                Ok(())
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

// MARK: FromRow
pub trait FromRow: Sized {
    fn from_row(statement: *mut sqlite3_stmt) -> Result<Self>;
}

impl FromRow for () {
    fn from_row(_statement: *mut sqlite3_stmt) -> Result<Self> {
        Ok(())
    }
}

impl<T> FromRow for T
where
    T: TryFrom<Value>,
{
    fn from_row(statement: *mut sqlite3_stmt) -> Result<Self> {
        T::try_from(Value::read_from_statement(statement, 0)?)
            .map_err(|_| Error::new("Can't convert to value"))
    }
}

// MARK: Statement
pub struct Statement<T> {
    statement: *mut sqlite3_stmt,
    _maker: PhantomData<T>,
}

impl<T> Statement<T> {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self {
            statement,
            _maker: PhantomData,
        }
    }

    pub fn bind(&mut self, params: impl Bind) -> Result<()> {
        params.bind(self.statement)
    }

    pub fn reset(&mut self) -> Result<()> {
        unsafe { sqlite3_reset(self.statement) };
        Ok(())
    }
}

impl<T> Iterator for Statement<T>
where
    T: FromRow,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { sqlite3_step(self.statement) };
        if result == SQLITE_ROW {
            Some(T::from_row(self.statement))
        } else if result == SQLITE_DONE {
            None
        } else {
            Some(Err(Error::new("Can't step statement")))
        }
    }
}

impl<T> Drop for Statement<T> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.statement) };
    }
}
