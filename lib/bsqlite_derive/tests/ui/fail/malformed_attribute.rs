/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite_derive::FromRow;

#[derive(FromRow)]
struct User {
    #[sqlite(rename = 42)]
    name: String,
}

fn main() {}
