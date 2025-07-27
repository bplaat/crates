/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [rust-embed-impl](https://crates.io/crates/rust-embed-impl) crate

#![forbid(unsafe_code)]

use std::path::Path;
use std::{env, fs};

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Lit, Meta, parse_macro_input};

/// [Embed] derive
#[proc_macro_derive(Embed, attributes(folder))]
pub fn validate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Extract the #[folder = "..."] attribute
    let mut folder_path = None;
    for attr in input.attrs {
        if attr.path().is_ident("folder") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(expr) = &meta.value {
                    if let Lit::Str(lit_str) = &expr.lit {
                        folder_path = Some(lit_str.value());
                        break;
                    }
                }
            }
            panic!("Invalid #[folder = \"...\"] attribute");
        }
    }
    let mut folder_path = match folder_path {
        Some(path) => path,
        None => panic!("Missing #[folder = \"...\"] attribute"),
    };

    // Replace $ENV_VAR in folder_path with their values from the environment
    #[cfg(feature = "interpolate-folder-path")]
    for (key, value) in env::vars() {
        let pattern = format!("${key}");
        if folder_path.contains(&pattern) {
            folder_path = folder_path.replace(&pattern, &value);
        }
    }

    // If the resolved_folder_path is still relative, make it absolute using CARGO_MANIFEST_DIR
    folder_path = if Path::new(&folder_path).is_relative() {
        format!(
            "{}/{}",
            env::var("CARGO_MANIFEST_DIR").expect("Should be some"),
            folder_path
        )
    } else {
        folder_path
    };

    // Recursively index all files from the folder
    fn visit_dir(dir: &Path, files: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("Can't read directory") {
            let path = entry.expect("Should be some").path();
            if path.is_dir() {
                visit_dir(&path, files);
            } else if path.is_file() {
                files.push(path.display().to_string());
            }
        }
    }
    let mut files = Vec::new();
    visit_dir(Path::new(&folder_path), &mut files);

    // Function to convert file paths to const names
    let to_const_name = |path: &str| {
        let mut file_path = path
            .strip_prefix(&folder_path)
            .expect("Should be relative path")
            .replace("\\", "/");
        if file_path.starts_with('/') {
            file_path = file_path[1..].to_string();
        }
        let const_name = format!(
            "_rust_embed_{}",
            file_path.to_lowercase().replace(['/', '.', '-'], "_")
        );
        let const_ident = syn::Ident::new(&const_name, proc_macro2::Span::call_site());
        (file_path.to_string(), const_ident)
    };

    // Create consts for all files
    let embed_files: Vec<_> = files
        .iter()
        .map(|path: &String| {
            let (_, const_ident) = to_const_name(path);
            quote! {
                const #const_ident: &[u8] = include_bytes!(#path);
            }
        })
        .collect();

    // Create a mapping of file paths to consts
    let embed_mapping: Vec<_> = files
        .iter()
        .map(|path| {
            let (file_path, const_ident) = to_const_name(path);
            quote! {
                #file_path => Some(rust_embed::EmbeddedFile {
                    data: std::borrow::Cow::Borrowed(#const_ident),
                }),
            }
        })
        .collect();

    TokenStream::from(quote! {
        #(#embed_files)*

        impl #name {
            fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
                <Self as rust_embed::RustEmbed>::get(file_path)
            }
        }

        impl rust_embed::RustEmbed for #name {
            fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
                match file_path {
                    #(#embed_mapping)*
                    _ => None,
                }
            }
        }
    })
}
