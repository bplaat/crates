/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) mod cx;
pub(crate) mod java;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Rule {
    CxCommon,
    C,
    Cpp,
    Objc,
    Objcpp,
    Ld,
    Bundle,
    Java,
    Jar,
}
