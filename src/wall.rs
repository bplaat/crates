/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::str::FromStr;

use web_sys::CanvasRenderingContext2d;

use crate::consts::*;

// MARK: Brick
pub(crate) struct Brick {
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
pub(crate) enum BondType {
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

pub(crate) struct Wall {
    pub(crate) width: f64,
    pub(crate) height: f64,
    bricks: Vec<Brick>,
}

impl Wall {
    pub(crate) fn new(width: f64, height: f64, bond: BondType) -> Self {
        let rows = (height / (BRICK_HEIGHT + BRICK_HEAD_JOINT)).ceil() as usize;

        let mut bricks = Vec::new();
        let mut y = height - BRICK_HEIGHT;
        for j in 0..rows {
            match bond {
                // MARK: Stretcher bond
                BondType::Stretcher => {
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Start half brick
                        bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                        x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;

                        // Normal bricks
                        while x + BRICK_WIDTH <= width {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }
                    } else {
                        // Normal bricks
                        while x + BRICK_WIDTH <= width - BRICK_HALF_WIDTH {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }

                        // End half brick
                        bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                    }
                }

                // MARK: Header bond
                BondType::Header => {
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Half bricks
                        while x + BRICK_HALF_WIDTH <= width {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }
                    } else {
                        // Start two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Half bricks
                        while x + BRICK_HALF_WIDTH <= width - BRICK_TWO_THIRDS_WIDTH {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }

                        // End two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                    }
                }

                // MARK: English bond
                BondType::English => {
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Normal bricks
                        while x + BRICK_WIDTH <= width {
                            bricks.push(Brick::new(x, y, BRICK_WIDTH));
                            x += BRICK_WIDTH + BRICK_BED_JOINT;
                        }

                        // Half brick at the end if space allows
                        if x + BRICK_HALF_WIDTH <= width {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                        }
                    } else {
                        // Start two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Half bricks
                        while x + BRICK_HALF_WIDTH <= width - BRICK_TWO_THIRDS_WIDTH {
                            bricks.push(Brick::new(x, y, BRICK_HALF_WIDTH));
                            x += BRICK_HALF_WIDTH + BRICK_BED_JOINT;
                        }

                        // End two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                    }
                }

                // MARK: Flemish bond
                BondType::Flemish => {
                    let mut x = 0.0;
                    if j % 2 == 0 {
                        // Alternate normal and half bricks
                        let mut is_normal = true;
                        while x < width {
                            let brick_width = if is_normal {
                                BRICK_WIDTH
                            } else {
                                BRICK_HALF_WIDTH
                            };
                            if x + brick_width > width {
                                break;
                            }
                            bricks.push(Brick::new(x, y, brick_width));
                            x += brick_width + BRICK_BED_JOINT;
                            is_normal = !is_normal;
                        }
                    } else {
                        // Start with two thirds brick
                        bricks.push(Brick::new(x, y, BRICK_TWO_THIRDS_WIDTH));
                        x += BRICK_TWO_THIRDS_WIDTH + BRICK_BED_JOINT;

                        // Alternate normal and half bricks
                        let mut is_normal = true;
                        while x + BRICK_TWO_THIRDS_WIDTH < width {
                            let brick_width = if is_normal {
                                BRICK_WIDTH
                            } else {
                                BRICK_HALF_WIDTH
                            };
                            if x + brick_width > width - BRICK_TWO_THIRDS_WIDTH {
                                break;
                            }
                            bricks.push(Brick::new(x, y, brick_width));
                            x += brick_width + BRICK_BED_JOINT;
                            is_normal = !is_normal;
                        }

                        // End with two thirds brick
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

    pub(crate) fn draw(&self, context: &CanvasRenderingContext2d) {
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
