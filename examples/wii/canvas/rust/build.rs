/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[path = "../../../../tools/wii-toolchain/build.rs"]
mod wii;

fn main() {
    wii::build("../../../../tools/wii-toolchain");
}
