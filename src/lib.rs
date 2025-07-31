/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::str::FromStr;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlFormElement};

// MARK: Constants
const BRICK_BED_JOINT: f64 = 10.0; // Padding between bricks
const BRICK_HEAD_JOINT: f64 = 12.5; // Padding between rows
const BRICK_HALF_WIDTH: f64 = 100.0;
const BRICK_WIDTH: f64 = BRICK_HALF_WIDTH * 2.0 + BRICK_BED_JOINT;
const BRICK_TWO_THIRDS_WIDTH: f64 = BRICK_HALF_WIDTH * 1.5 + BRICK_BED_JOINT / 2.0;
const BRICK_HEIGHT: f64 = 50.0;

const STRIDE_WIDTH: f64 = 800.0; // Robot's horizontal reach
const STRIDE_HEIGHT: f64 = 1300.0; // Robot's vertical reach

// MARK: Brick
struct Brick {
    x: f64,
    y: f64,
    width: f64,
}

impl Brick {
    fn new(x: f64, y: f64, width: f64) -> Self {
        Brick { x, y, width }
    }

    fn draw(&self, context: &CanvasRenderingContext2d) {
        context.set_fill_style_str("red");
        context.fill_rect(self.x, self.y, self.width, BRICK_HEIGHT);
    }
}

// MARK: Wall
enum BondType {
    Stretcher, // Normal bricks
    Header,    // Half bricks
    English,   // Half bricks every alternating row
    Flemish,   // Half bricks alternating
}

impl FromStr for BondType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stretcher" => Ok(BondType::Stretcher),
            "header" => Ok(BondType::Header),
            "english" => Ok(BondType::English),
            "flemish" => Ok(BondType::Flemish),
            _ => Err(()),
        }
    }
}

struct Wall {
    width: f64,
    height: f64,
    bricks: Vec<Brick>,
}

impl Wall {
    fn new(width: f64, height: f64, bond: BondType) -> Self {
        // FIXME: Make brick filling algo more flexible
        let rows = (height / (BRICK_HEIGHT + BRICK_HEAD_JOINT)).ceil() as usize;
        let mut bricks = Vec::new();
        let mut y = height - BRICK_HEIGHT;
        for j in 0..rows {
            match bond {
                // MARK: Stretcher bond
                BondType::Stretcher => {
                    let cols = (width / (BRICK_WIDTH + BRICK_BED_JOINT)).ceil() as usize;
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Normal bricks
                        for _ in 0..(cols - 1) {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }

                        // End half brick
                        bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                    } else {
                        // Start half brick
                        bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                        x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;

                        // Normal bricks
                        for _ in 0..(cols - 1) {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }
                    }
                }

                // MARK: Header bond
                BondType::Header => {
                    let cols = (width / (BRICK_HALF_WIDTH + BRICK_BED_JOINT)).ceil() as usize;
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Half bricks
                        for _ in 0..cols {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }
                    } else {
                        // Start two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));

                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Half bricks
                        for _ in 0..(cols - 3) {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }

                        // End two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                    }
                }

                // MARK: English bond
                BondType::English => {
                    let cols = ((width - BRICK_TWO_THIRDS_WIDTH) / (BRICK_WIDTH + BRICK_BED_JOINT))
                        .ceil() as usize;
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Normal bricks
                        for _ in 0..cols {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }

                        // Half brick at the end
                        bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                    } else {
                        // Start two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));

                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Half bricks
                        for _ in 0..((cols - 1) * 2) {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }

                        // End two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                    }
                }

                // MARK: Flemish bond
                BondType::Flemish => {
                    let cols = ((width - BRICK_TWO_THIRDS_WIDTH) / (BRICK_WIDTH + BRICK_BED_JOINT))
                        .ceil() as usize;
                    let mut x = 0.0;
                    let len = cols * 3 / 2 - 1;
                    if j % 2 == 0 {
                        // Normal or half brick
                        for i in 0..len {
                            let width = if i % 2 == 0 {
                                BRICK_WIDTH
                            } else {
                                BRICK_HALF_WIDTH
                            };
                            bricks.push(Brick::new(x, y, width));
                            x += width + BRICK_BED_JOINT;
                        }
                    } else {
                        // Start two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Normal or half brick
                        for i in 0..len - 2 {
                            let width = if i % 2 == 0 {
                                BRICK_WIDTH
                            } else {
                                BRICK_HALF_WIDTH
                            };
                            bricks.push(Brick::new(x, y, width));
                            x += width + BRICK_BED_JOINT;
                        }

                        // End two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                    }
                }
            }

            y -= BRICK_HEIGHT + BRICK_HEAD_JOINT;
        }
        Wall {
            width,
            height,
            bricks,
        }
    }

    fn draw(&self, context: &CanvasRenderingContext2d) {
        for brick in &self.bricks {
            brick.draw(context);
        }

        // Draw wall outline
        context.set_stroke_style_str("green");
        context.set_line_width(5.0);
        context.stroke_rect(0.0, 0.0, self.width, self.height);

        // Draw robot's reach
        context.set_stroke_style_str("blue");
        context.stroke_rect(
            0.0,
            self.height - STRIDE_HEIGHT,
            STRIDE_WIDTH,
            STRIDE_HEIGHT,
        );
    }
}

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
