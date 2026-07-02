/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse_macro_input;

struct MigrationsInput {
    db: syn::Expr,
    path: syn::LitStr,
}

impl Parse for MigrationsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let db = input.parse()?;
        let _comma: syn::Token![,] = input.parse()?;
        let path = input.parse()?;
        Ok(Self { db, path })
    }
}

pub(crate) fn run_migrations(input: TokenStream) -> TokenStream {
    let MigrationsInput { db, path } = parse_macro_input!(input as MigrationsInput);
    let path_str = path.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let dir = std::path::Path::new(&manifest_dir).join(&path_str);

    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("migrations!: cannot read '{}': {}", dir.display(), e))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().map_or(false, |e| e == "sql")
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s.starts_with('V') || s.starts_with('v'))
        })
        .collect();

    paths.sort_by_key(|p| migration_version(&p.file_stem().unwrap_or_default().to_string_lossy()));

    let items = paths.iter().map(|p| {
        let name = p
            .file_stem()
            .expect("Invalid migration file")
            .to_string_lossy()
            .into_owned();
        let abs = p.to_str().expect("non-UTF-8 migration path");
        quote! {
            bsqlite::Migration { name: #name, sql: include_str!(#abs) }
        }
    });

    TokenStream::from(quote! {
        (#db).migration(&[#(#items),*])
    })
}

fn migration_version(name: &str) -> u32 {
    let s = name.trim_start_matches(['V', 'v']);
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    s[..end].parse().unwrap_or(0)
}
