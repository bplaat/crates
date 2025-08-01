/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![forbid(unsafe_code)]

use std::sync::RwLock;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlFormElement, KeyboardEvent};

use crate::wall::{BondType, Wall};

mod brick;
mod consts;
mod wall;

static WALL: RwLock<Option<Wall>> = RwLock::new(None);

fn init_wall(
    context: &CanvasRenderingContext2d,
    wall_width_input: &web_sys::HtmlInputElement,
    wall_height_input: &web_sys::HtmlInputElement,
    wall_bond_select: &web_sys::HtmlSelectElement,
) {
    let wall = Wall::new(
        wall_width_input.value().parse().unwrap(),
        wall_height_input.value().parse().unwrap(),
        wall_bond_select.value().parse::<BondType>().unwrap(),
    );
    wall.draw(context);
    *WALL.write().unwrap() = Some(wall);
}

#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    // MARK: Elements
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

    // Actions
    let next_brick_button = document
        .get_element_by_id("next-brick-button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()?;
    let fill_bricks_button = document
        .get_element_by_id("fill-bricks-button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()?;

    // MARK: Init wall
    {
        let wall_width_input = wall_width_input.clone();
        let wall_height_input = wall_height_input.clone();
        let wall_bond_select = wall_bond_select.clone();
        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();

            init_wall(
                &context,
                &wall_width_input,
                &wall_height_input,
                &wall_bond_select,
            );
        }) as Box<dyn FnMut(_)>);
        wall_form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let wall_width_input = wall_width_input.clone();
        let wall_height_input = wall_height_input.clone();
        let wall_bond_select = wall_bond_select.clone();
        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();

            init_wall(
                &context,
                &wall_width_input,
                &wall_height_input,
                &wall_bond_select,
            );
        }) as Box<dyn FnMut(_)>);
        wall_form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    init_wall(
        &context,
        &wall_width_input,
        &wall_height_input,
        &wall_bond_select,
    );

    // MARK: Next brick
    {
        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            let mut wall = WALL.write().unwrap();
            if let Some(ref mut wall) = *wall {
                wall.next_brick();

                wall.draw(&context);
            }
        }) as Box<dyn FnMut(_)>);
        next_brick_button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                event.prevent_default();

                let mut wall = WALL.write().unwrap();
                if let Some(ref mut wall) = *wall {
                    wall.next_brick();
                    wall.draw(&context);
                }
            }
        }) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // MARK: Fill bricks
    {
        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();

            let mut wall = WALL.write().unwrap();
            if let Some(ref mut wall) = *wall {
                wall.fill_bricks();
                wall.draw(&context);
            }
        }) as Box<dyn FnMut(_)>);
        fill_bricks_button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}
