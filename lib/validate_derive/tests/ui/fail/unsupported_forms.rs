/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use validate_derive::Validate;

#[derive(Validate)]
enum Choice {
    One,
}

#[derive(Validate)]
struct Tuple(String);

fn main() {}
