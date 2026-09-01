/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use from_derive::FromEnum;

enum OtherState {
    Ready,
}

#[derive(FromEnum)]
#[from_enum(OtherState, only_from, only_into)]
enum State {
    Ready,
}

fn main() {}
