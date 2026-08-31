/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate self as bsql;

use bsql_derive::{FromRow, FromValue};

enum Value {
    Null,
    Integer(i64),
    Text(String),
}

struct ValueError;
impl ValueError {
    fn new(_: impl Into<String>) -> Self {
        Self
    }
}

struct StatementError;
struct RawStatement;
impl RawStatement {
    fn bind_value(&mut self, _: i32, _: Value) -> Result<(), StatementError> {
        Ok(())
    }

    fn column_value(&mut self, _: i32) -> Value {
        Value::Null
    }
}

trait Bind {
    fn bind(self, statement: &mut RawStatement) -> Result<(), StatementError>;
}

trait FromRow: Sized {
    fn from_row(statement: &mut RawStatement) -> Result<Self, ValueError>;
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl TryFrom<Value> for String {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Text(value) => Ok(value),
            _ => Err(ValueError),
        }
    }
}

impl From<Option<String>> for Value {
    fn from(value: Option<String>) -> Self {
        value.map_or(Self::Null, Self::Text)
    }
}

impl TryFrom<Value> for Option<String> {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(None),
            Value::Text(value) => Ok(Some(value)),
            _ => Err(ValueError),
        }
    }
}

#[derive(Default, FromRow)]
struct User<T>
where
    T: Default + Into<Value> + TryFrom<Value, Error = ValueError>,
{
    #[sql(rename = "display_name")]
    name: T,
    nickname: Option<String>,
    #[sql(skip)]
    cached: bool,
}

#[derive(FromValue)]
enum Status {
    Active = 1,
    Disabled = 2,
}

fn main() {
    let _: Value = Status::Active.into();
    let _: Result<Status, ValueError> = Value::Integer(2).try_into();
    assert_eq!(User::<String>::columns(), "display_name, nickname");
    assert_eq!(User::<String>::values(), "?, ?");
}
