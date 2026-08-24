/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate self as rust_embed;

use std::borrow::Cow;
use rust_embed_impl::Embed;

struct EmbeddedFile {
    data: Cow<'static, [u8]>,
}

trait RustEmbed {
    fn get(file_path: &str) -> Option<EmbeddedFile>;
    fn iter() -> impl Iterator<Item = Cow<'static, str>> + 'static;
}

#[derive(Embed)]
#[folder = "../../../../lib/rust-embed-impl/tests/ui/assets"]
struct Assets;

fn main() {
    let _ = Assets::get("hello.txt");
    let _ = Assets::iter();
}
