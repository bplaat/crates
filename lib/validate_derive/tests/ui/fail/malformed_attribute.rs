/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use validate_derive::Validate;

#[derive(Validate)]
struct Form {
    #[validate(length(min =))]
    name: String,
}

fn main() {}
