/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unused)]

#[cfg_attr(target_env = "msvc", link(name = "advapi32"))]
unsafe extern "system" {}

include!(concat!(env!("OUT_DIR"), "/webview2_bindings.rs"));
