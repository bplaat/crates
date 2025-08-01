/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use web_sys::CanvasRenderingContext2d;

use crate::consts::BRICK_HEIGHT;

#[derive(Clone)]
pub(crate) struct Brick {
    x: f64,
    y: f64,
    width: f64,
    build_stride: Option<usize>,
}

impl Brick {
    pub(crate) fn new(x: f64, y: f64, width: f64) -> Self {
        Brick {
            x,
            y,
            width,
            build_stride: None,
        }
    }

    pub(crate) fn x(&self) -> f64 {
        self.x
    }

    pub(crate) fn y(&self) -> f64 {
        self.y
    }

    pub(crate) fn width(&self) -> f64 {
        self.width
    }

    pub(crate) fn is_build(&self) -> bool {
        self.build_stride.is_some()
    }

    pub(crate) fn build(&mut self, stride: usize) {
        self.build_stride = Some(stride);
    }

    pub(crate) fn draw(&self, context: &CanvasRenderingContext2d) {
        // Draw  brick
        if let Some(stride) = self.build_stride {
            let color = format!("hsl({}, 70%, 30%)", 240 - (stride as i32 * 37 % 240));
            context.set_fill_style_str(&color);
        } else {
            context.set_fill_style_str("#faa");
        }
        context.fill_rect(self.x, self.y, self.width, BRICK_HEIGHT);

        // Draw build stride number
        if let Some(stride) = self.build_stride {
            context.set_font(&format!(
                "bold {}px sans-serif",
                (BRICK_HEIGHT * 0.75) as u32
            ));
            context.set_text_align("center");
            context.set_text_baseline("middle");
            context.set_fill_style_str("#fff");
            context
                .fill_text(
                    &stride.to_string(),
                    self.x + self.width / 2.0,
                    self.y + BRICK_HEIGHT / 2.0,
                )
                .unwrap();
        }
    }
}
