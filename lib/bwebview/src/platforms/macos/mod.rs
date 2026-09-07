/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(feature = "file_drop")]
mod file_drop;
mod headers;
mod webview;

pub(crate) use webview::PlatformWebview;
