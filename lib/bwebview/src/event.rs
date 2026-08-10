/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::{LogicalPoint, LogicalSize, Theme};

/// A close request that can be prevented by the event handler
#[derive(Clone)]
pub struct CloseRequest(Rc<Cell<bool>>);

impl CloseRequest {
    pub(crate) fn new() -> Self {
        Self(Rc::new(Cell::new(false)))
    }

    /// Prevent the window from closing
    pub fn prevent_close(&self) {
        self.0.set(true);
    }

    pub(crate) fn is_prevented(&self) -> bool {
        self.0.get()
    }
}

/// Window event
pub enum WindowEvent {
    /// Window create
    Create,
    /// Window move
    Move(LogicalPoint),
    /// Window resize
    Resize(LogicalSize),
    /// The effective light or dark system appearance changed.
    ThemeChange(Theme),
    /// Window close requested; closes normally unless prevented
    CloseRequested(CloseRequest),
    /// File dropped
    #[cfg(feature = "file_drop")]
    DroppedFile(PathBuf),
    /// macOS window fullscreen change
    #[cfg(target_os = "macos")]
    MacosFullscreenChange(bool),
}

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

/// Event
pub enum Event {
    /// Window event
    Window(WindowEvent),
    /// Webview event
    Webview(WebviewEvent),
    /// User event
    UserEvent(String),
    /// Files opened through macOS Launch Services
    #[cfg(target_os = "macos")]
    MacosOpenFiles(Vec<PathBuf>),
    /// A custom macOS menu item was selected
    #[cfg(all(target_os = "macos", feature = "menu"))]
    MacosMenuItem(String),
}
