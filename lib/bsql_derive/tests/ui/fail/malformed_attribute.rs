/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsql_derive::FromRow;

#[derive(FromRow)]
struct User {
    #[sql(rename = 42)]
    name: String,
}

fn main() {}
