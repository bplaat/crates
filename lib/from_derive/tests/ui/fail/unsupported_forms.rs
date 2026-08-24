/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use from_derive::{FromEnum, FromStruct};

enum OtherState {
    Ready(String),
}

#[derive(FromEnum)]
#[from_enum(OtherState)]
enum State {
    Ready(String),
}

struct OtherRecord(String);

#[derive(FromStruct)]
#[from_struct(OtherRecord)]
struct Record(String);

fn main() {}
