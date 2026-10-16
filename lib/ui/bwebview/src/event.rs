/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// Webview event
pub enum WebviewEvent {
    /// Page load start
    PageLoadStart,
    /// Page load finish
    PageLoadFinish,
    /// Page title change
    PageTitleChange(String),
    /// IPC message receive
    MessageReceive(String),
}
