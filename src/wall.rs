/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::str::FromStr;

use web_sys::CanvasRenderingContext2d;

use crate::brick::Brick;
use crate::consts::*;

// MARK: Bond type
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

// MARK: Wall
pub(crate) struct Wall {
    width: f64,
    height: f64,
    bricks: Vec<Brick>,
    robot_x: f64,
    robot_y: f64,
    current_stride: usize,
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
            robot_x: 0.0,
            robot_y: height - STRIDE_HEIGHT,
            current_stride: 1,
        }
    }

    // MARK: Next brick algorithm
    pub(crate) fn next_brick(&mut self) -> bool {
        // Clone bricks to avoid borrowing issues
        let bricks = self.bricks.clone();

        // Get all unbuild bricks
        let mut bricks_iter = self
            .bricks
            .iter_mut()
            .filter(|b| !b.is_build())
            .collect::<Vec<_>>();

        // Sort bricks so those within robot's reach come first
        bricks_iter.sort_by_key(|brick| {
            let in_reach = brick.x() >= self.robot_x
                && brick.x() + brick.width() <= self.robot_x + STRIDE_WIDTH
                && brick.y() >= self.robot_y
                && brick.y() + BRICK_HEIGHT <= self.robot_y + STRIDE_HEIGHT;
            if in_reach { 0 } else { 1 }
        });

        // Filter out all bricks that can't be build
        bricks_iter.retain(|brick| {
            // If brick is on the bottom row, it can always be built
            if brick.y() == self.height - BRICK_HEIGHT {
                return true;
            }

            // Check if all bricks below this brick are built
            let below_y = brick.y() + BRICK_HEIGHT + BRICK_HEAD_JOINT;
            let mut covered = 0.0;
            for b in &bricks {
                if b.y() == below_y
                    && b.is_build()
                    && b.x() < brick.x() + brick.width()
                    && b.x() + b.width() > brick.x()
                {
                    let left = brick.x().max(b.x());
                    let right = (brick.x() + brick.width()).min(b.x() + b.width());
                    covered += right - left + BRICK_HEAD_JOINT;
                }
            }
            covered >= brick.width()
        });

        if let Some(brick) = bricks_iter.first_mut() {
            // Check if brick is within robot's reach
            let in_reach = brick.x() >= self.robot_x
                && brick.x() + brick.width() <= self.robot_x + STRIDE_WIDTH
                && brick.y() >= self.robot_y
                && brick.y() + BRICK_HEIGHT <= self.robot_y + STRIDE_HEIGHT;

            // If not, move the robot to the bricks center
            if !in_reach {
                self.robot_x = (brick.x() + brick.width() / 2.0 - STRIDE_WIDTH / 2.0)
                    .clamp(0.0, self.width - STRIDE_WIDTH);
                self.robot_y = (brick.y() + BRICK_HEIGHT / 2.0 - STRIDE_HEIGHT / 2.0)
                    .clamp(0.0, self.height - STRIDE_HEIGHT);
                self.current_stride += 1;
            }

            // Build the brick
            brick.build(self.current_stride);
            true
        } else {
            false
        }
    }

    pub(crate) fn fill_bricks(&mut self) {
        while self.next_brick() {}
    }

    // MARK: Draw wall
    pub(crate) fn draw(&self, context: &CanvasRenderingContext2d) {
        let canvas = context.canvas().unwrap();

        // Scale canvas
        let scale = canvas.width() as f64 / (self.width * 1.2);
        context.reset();
        context.scale(scale, scale).unwrap();

        // Clear canvas
        context.clear_rect(
            0.0,
            0.0,
            canvas.width() as f64 * scale,
            canvas.height() as f64 * scale,
        );

        // Draw bricks
        for brick in &self.bricks {
            brick.draw(context);
        }

        // Draw wall outline and label
        context.set_stroke_style_str("green");
        context.set_line_width(5.0);
        context.stroke_rect(0.0, 0.0, self.width, self.height);
        context.set_fill_style_str("green");
        context.fill_rect(0.0, 0.0, 80.0, 30.0);
        context.set_fill_style_str("white");
        context.set_font("bold 20px sans-serif");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.fill_text("Wall", 80.0 / 2.0, 30.0 / 2.0).unwrap();

        // Draw robot's reach and label
        context.set_stroke_style_str("blue");
        context.stroke_rect(self.robot_x, self.robot_y, STRIDE_WIDTH, STRIDE_HEIGHT);
        context.set_fill_style_str("blue");
        context.fill_rect(self.robot_x, self.robot_y, 80.0, 30.0);
        context.set_fill_style_str("white");
        context.set_font("bold 20px sans-serif");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context
            .fill_text(
                "Robot",
                self.robot_x + 80.0 / 2.0,
                self.robot_y + 30.0 / 2.0,
            )
            .unwrap();
    }
}
