/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite_derive::{FromRow, FromValue};

#[derive(FromRow)]
struct Row(String);

#[derive(FromValue)]
enum Status {
    MissingDiscriminant,
}

#[derive(FromValue)]
struct NotAnEnum;

fn main() {}
