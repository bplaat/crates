/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().expect("Should have window");
    let document = window.document().expect("Should have document");

    let canvas = document
        .get_element_by_id("canvas")
        .expect("Should find canvas element")
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")
        .expect("Should get 2d context")
        .expect("Should have 2d context")
        .dyn_into::<CanvasRenderingContext2d>()?;

    context.set_fill_style_str("red");
    context.fill_rect(50.0, 50.0, 100.0, 100.0);

    Ok(())
}
