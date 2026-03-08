/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::platforms::PlatformFileDialog;

pub(crate) struct FileDialogFilter {
    pub(crate) name: String,
    pub(crate) extensions: Vec<String>,
}

pub(crate) trait FileDialogInterface {
    fn pick_file(dialog: FileDialog) -> Option<std::path::PathBuf>;
    fn pick_files(dialog: FileDialog) -> Option<Vec<std::path::PathBuf>>;
    fn save_file(dialog: FileDialog) -> Option<std::path::PathBuf>;
}

/// File dialog builder
#[derive(Default)]
pub struct FileDialog {
    pub(crate) title: Option<String>,
    pub(crate) directory: Option<std::path::PathBuf>,
    pub(crate) filename: Option<String>,
    pub(crate) filters: Vec<FileDialogFilter>,
}

impl FileDialog {
    /// Create new file dialog
    pub fn new() -> Self {
        Self::default()
    }

    /// Set dialog title
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = Some(title.as_ref().to_string());
        self
    }

    /// Set starting directory
    pub fn set_directory(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.directory = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set default file name (used for save dialogs)
    pub fn set_file_name(mut self, filename: impl AsRef<str>) -> Self {
        self.filename = Some(filename.as_ref().to_string());
        self
    }

    /// Add a file filter (name + list of extensions without dot)
    pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[impl AsRef<str>]) -> Self {
        self.filters.push(FileDialogFilter {
            name: name.as_ref().to_string(),
            extensions: extensions.iter().map(|e| e.as_ref().to_string()).collect(),
        });
        self
    }

    /// Open a single-file picker dialog
    pub fn pick_file(self) -> Option<std::path::PathBuf> {
        PlatformFileDialog::pick_file(self)
    }

    /// Open a multi-file picker dialog
    pub fn pick_files(self) -> Option<Vec<std::path::PathBuf>> {
        PlatformFileDialog::pick_files(self)
    }

    /// Open a save-file dialog
    pub fn save_file(self) -> Option<std::path::PathBuf> {
        PlatformFileDialog::save_file(self)
    }
}
