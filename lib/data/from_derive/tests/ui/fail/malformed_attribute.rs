/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use from_derive::FromStruct;

struct Other {
    value: String,
}

#[derive(FromStruct)]
#[from_struct = "Other"]
struct Record {
    value: String,
}

fn main() {}
