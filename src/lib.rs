/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// MARK: Brick
const BRICK_WIDTH: f64 = 210.0;
const BRICK_HALF_WIDTH: f64 = 100.0;
const BRICK_HEIGHT: f64 = 50.0;
const BRICK_HEAD_JOINT: f64 = 10.0; // Padding between bricks
const BRICK_BED_JOINT: f64 = 12.5; // Padding between rows

enum BrickType {
    Normal,
    Half,
}

struct Brick {
    r#type: BrickType,
    x: f64,
    y: f64,
}

impl Brick {
    fn new(r#type: BrickType, x: f64, y: f64) -> Self {
        Brick { r#type, x, y }
    }

    fn draw(&self, context: &CanvasRenderingContext2d) {
        context.set_fill_style_str("red");
        let width = match self.r#type {
            BrickType::Normal => BRICK_WIDTH,
            BrickType::Half => BRICK_HALF_WIDTH,
        };
        context.fill_rect(self.x, self.y, width, BRICK_HEIGHT);
    }
}

// MARK: Wall
struct Wall {
    width: f64,
    height: f64,
    bricks: Vec<Brick>,
}

impl Wall {
    fn new(width: f64, height: f64) -> Self {
        let cols = (width / (BRICK_WIDTH + BRICK_HEAD_JOINT)).ceil() as usize;
        let rows = (height / (BRICK_HEIGHT + BRICK_BED_JOINT)).ceil() as usize;

        let mut bricks = Vec::new();
        for j in 0..rows {
            let y = j as f64 * (BRICK_HEIGHT + BRICK_BED_JOINT) + BRICK_BED_JOINT;
            // On odd rows, start and end with a half brick
            if j % 2 == 0 {
                let mut x = 0.0;

                // Normal bricks
                for _ in 0..(cols - 1) {
                    let brick = Brick::new(BrickType::Normal, x, y);
                    bricks.push(brick);
                    x += BRICK_WIDTH + BRICK_HEAD_JOINT;
                }

                // End half brick
                let end_brick = Brick::new(BrickType::Half, x, y);
                bricks.push(end_brick);
            } else {
                let mut x = 0.0;

                // Start half brick
                let start_brick = Brick::new(BrickType::Half, x, y);
                bricks.push(start_brick);

                x += BRICK_HALF_WIDTH + BRICK_HEAD_JOINT;

                // Normal bricks
                for _ in 0..(cols - 1) {
                    let brick = Brick::new(BrickType::Normal, x, y);
                    bricks.push(brick);
                    x += BRICK_WIDTH + BRICK_HEAD_JOINT;
                }
            }
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
    }
}

// MARK: Main
#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    // Get canvas context
    let window = web_sys::window().expect("Should have window");
    let document = window.document().expect("Should have document");

    let canvas = document
        .get_element_by_id("canvas")
        .expect("Should find canvas element")
        .dyn_into::<HtmlCanvasElement>()?;
    let context = canvas
        .get_context("2d")?
        .expect("Should have 2d context")
        .dyn_into::<CanvasRenderingContext2d>()?;

    // FIXME: Add scaling controls
    context.scale(0.35, 0.35)?;

    // Create wall and draw
    const ASSIGNMENT_WALL_WIDTH: f64 = 2300.0;
    const ASSIGNMENT_WALL_HEIGHT: f64 = 2000.0;
    let wall = Wall::new(ASSIGNMENT_WALL_WIDTH, ASSIGNMENT_WALL_HEIGHT);
    context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    wall.draw(&context);

    Ok(())
}
