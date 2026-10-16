/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{KeyCode, Modifiers};

/// Typed macOS menu keyboard accelerator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Accelerator {
    pub(crate) modifiers: Modifiers,
    pub(crate) key: KeyCode,
}

impl Accelerator {
    /// Create an accelerator from modifiers and a physical key code
    pub const fn new(modifiers: Modifiers, key: KeyCode) -> Self {
        Self { modifiers, key }
    }
}

/// macOS menu item builder
pub struct MenuItem {
    pub(crate) title: String,
    pub(crate) action: String,
    pub(crate) accelerator: Option<Accelerator>,
}

impl MenuItem {
    /// Create a menu item with a title and stable action identifier
    pub fn new(title: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: action.into(),
            accelerator: None,
        }
    }

    /// Set the item's keyboard accelerator
    pub const fn accelerator(mut self, accelerator: Accelerator) -> Self {
        self.accelerator = Some(accelerator);
        self
    }
}

pub(crate) enum MenuBuilderEntry {
    Item(MenuItem),
    Separator,
}

/// Builder for one macOS menu
pub struct MenuBuilder {
    pub(crate) title: String,
    pub(crate) entries: Vec<MenuBuilderEntry>,
}

impl MenuBuilder {
    /// Create an empty named menu
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            entries: Vec::new(),
        }
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuBuilderEntry::Item(item));
        self
    }

    /// Add a separator
    pub fn separator(mut self) -> Self {
        self.entries.push(MenuBuilderEntry::Separator);
        self
    }
}

/// Builds custom macOS entries merged by name into bwindow's default menu bar.
#[derive(Default)]
pub struct MenuBarBuilder {
    pub(crate) menus: Vec<MenuBuilder>,
}

impl MenuBarBuilder {
    /// Create an empty menu bar builder
    pub const fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Add or extend a menu
    pub fn menu(mut self, menu: MenuBuilder) -> Self {
        self.menus.push(menu);
        self
    }
}
