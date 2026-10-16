/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate self as validate;

use std::collections::HashMap;
use std::marker::PhantomData;
use validate_derive::Validate;

#[derive(Default)]
struct Report(HashMap<String, Vec<String>>);

impl Report {
    fn new() -> Self {
        Self::default()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn insert_error(&mut self, field: impl Into<String>, error: impl Into<String>) {
        self.0.entry(field.into()).or_default().push(error.into());
    }
}

struct Error(String);

impl Error {
    fn message(&self) -> &str {
        &self.0
    }
}

trait Validate {
    type Context;

    fn validate_with(&self, context: &Self::Context) -> Result<(), Report>;
}

struct Context;

fn allowed(value: &String, _context: &Context) -> Result<(), Error> {
    if value.is_empty() {
        Err(Error("empty".into()))
    } else {
        Ok(())
    }
}

#[derive(Validate)]
#[validate(context(Context))]
struct Form<'a, T> {
    #[validate(length(min = 2), ascii, custom(allowed))]
    name: String,
    #[validate(range(min = 1, max = 10))]
    count: Option<i32>,
    marker: PhantomData<&'a T>,
}

fn main() {
    let form = Form::<u8> {
        name: "ok".into(),
        count: None,
        marker: PhantomData,
    };
    let _ = form.validate_with(&Context);
}
