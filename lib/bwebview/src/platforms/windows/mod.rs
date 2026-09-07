/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

mod callback;
#[cfg(feature = "file_drop")]
mod file_drop;
mod headers;
mod loader;
mod webview;

pub(crate) use webview::PlatformWebview;
