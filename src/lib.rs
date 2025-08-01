/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlFormElement};

use crate::wall::{BondType, Wall};

mod consts;
mod wall;

// MARK: Main
#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Get canvas context
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Wall form
    let wall_form = document
        .get_element_by_id("wall-form")
        .unwrap()
        .dyn_into::<HtmlFormElement>()?;
    let wall_width_input = document
        .get_element_by_id("wall-width")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()?;
    let wall_height_input = document
        .get_element_by_id("wall-height")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()?;
    let wall_bond_select = document
        .get_element_by_id("wall-bond")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?;
    {
        let wall_width_input = wall_width_input.clone();
        let wall_height_input = wall_height_input.clone();
        let wall_bond_select = wall_bond_select.clone();
        let canvas = canvas.clone();
        let context = context.clone();

        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();

            // Create wall and draw
            let wall = Wall::new(
                wall_width_input.value().parse().unwrap(),
                wall_height_input.value().parse().unwrap(),
                wall_bond_select.value().parse::<BondType>().unwrap(),
            );

            // Scale the canvas to fit the wall
            let scale = canvas.width() as f64 / (wall.width * 1.2);
            context.reset();
            context.scale(scale, scale).unwrap();

            // Draw wall
            context.clear_rect(
                0.0,
                0.0,
                canvas.width() as f64 * scale,
                canvas.height() as f64 * scale,
            );
            wall.draw(&context);
        }) as Box<dyn FnMut(_)>);
        wall_form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Create wall and draw
    let wall = Wall::new(
        wall_width_input.value().parse().unwrap(),
        wall_height_input.value().parse().unwrap(),
        wall_bond_select.value().parse::<BondType>().unwrap(),
    );

    // Scale the canvas to fit the wall
    let scale = canvas.width() as f64 / (wall.width * 1.2);
    context.reset();
    context.scale(scale, scale).unwrap();

    // Draw wall
    context.clear_rect(
        0.0,
        0.0,
        canvas.width() as f64 * scale,
        canvas.height() as f64 * scale,
    );
    wall.draw(&context);

    Ok(())
}
