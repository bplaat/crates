/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) const LOCK_EX: i32 = 2;
pub(crate) const LOCK_NB: i32 = 4;
unsafe extern "C" {
    pub(crate) fn flock(fd: i32, op: i32) -> i32;
}
