/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;

mod from_row;
mod from_value;
mod run_migrations;

/// Run all pending `V*.sql` migrations from a folder against a connection.
/// Files are sorted by version number parsed from the `V<n>__` prefix.
#[proc_macro]
pub fn run_migrations(input: TokenStream) -> TokenStream {
    run_migrations::run_migrations(input)
}

/// [FromRow] derive for structs
#[proc_macro_derive(FromRow, attributes(sqlite))]
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    from_row::from_row_derive(input)
}

/// [FromValue] derive for enums
#[proc_macro_derive(FromValue)]
pub fn from_value_derive(input: TokenStream) -> TokenStream {
    from_value::from_value_derive(input)
}
