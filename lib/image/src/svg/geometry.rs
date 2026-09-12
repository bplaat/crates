/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::values::number;
use super::xml::Element;
use crate::vector::path_bounds;
use crate::{
    DrawCommand, LineCap, LineJoin, MaskType, PathSegment, Point, Rect, StrokeStyle, Transform,
    VectorDecodeError, VectorImage,
};

pub(super) fn transform_bounds(bounds: Rect, transform: Transform) -> Rect {
    let points = [
        transform.map(Point {
            x: bounds.x,
            y: bounds.y,
        }),
        transform.map(Point {
            x: bounds.x + bounds.width,
            y: bounds.y,
        }),
        transform.map(Point {
            x: bounds.x,
            y: bounds.y + bounds.height,
        }),
        transform.map(Point {
            x: bounds.x + bounds.width,
            y: bounds.y + bounds.height,
        }),
    ];
    let mut output = Rect::from_points(points[0], points[0]);
    for point in &points[1..] {
        output.include(*point);
    }
    output
}

pub(super) fn inverse_transform(transform: Transform) -> Option<Transform> {
    let determinant = transform.a * transform.d - transform.b * transform.c;
    if determinant.abs() <= f64::EPSILON {
        return None;
    }
    Some(Transform {
        a: transform.d / determinant,
        b: -transform.b / determinant,
        c: -transform.c / determinant,
        d: transform.a / determinant,
        e: (transform.c * transform.f - transform.d * transform.e) / determinant,
        f: (transform.b * transform.e - transform.a * transform.f) / determinant,
    })
}

pub(super) fn commands_bounds(commands: &[DrawCommand]) -> Option<Rect> {
    commands
        .iter()
        .filter_map(command_bounds)
        .reduce(union_rect)
}

pub(super) fn commands_object_bounds(
    image: &VectorImage,
    commands: &[DrawCommand],
    document_to_object: Transform,
) -> Option<Rect> {
    commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill {
                path, transform, ..
            }
            | DrawCommand::Stroke {
                path, transform, ..
            } => path_bounds(image.path(*path))
                .map(|bounds| transform_bounds(bounds, transform.then(document_to_object))),
            DrawCommand::PushScope { .. } | DrawCommand::PopScope => None,
        })
        .reduce(union_rect)
}

const fn command_bounds(command: &DrawCommand) -> Option<Rect> {
    match command {
        DrawCommand::PushScope { bounds, .. }
        | DrawCommand::Fill { bounds, .. }
        | DrawCommand::Stroke { bounds, .. } => Some(*bounds),
        DrawCommand::PopScope => None,
    }
}

pub(super) fn union_rect(a: Rect, b: Rect) -> Rect {
    let max_x = (a.x + a.width).max(b.x + b.width);
    let max_y = (a.y + a.height).max(b.y + b.height);
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect {
        x,
        y,
        width: max_x - x,
        height: max_y - y,
    }
}

pub(super) fn object_bbox_length(value: &str) -> Result<f64, VectorDecodeError> {
    if let Some(value) = value.trim().strip_suffix('%') {
        Ok(number(value)? / 100.0)
    } else {
        number(value)
    }
}

pub(super) fn parse_mask_type(element: &Element<'_>) -> Result<MaskType, VectorDecodeError> {
    let mut mask_type = match element.attr("mask-type").unwrap_or("luminance") {
        "alpha" => MaskType::Alpha,
        "luminance" => MaskType::Luminance,
        _ => return Err(VectorDecodeError::InvalidData),
    };
    if let Some(style) = element.attr("style") {
        for declaration in style.split(';') {
            let Some((name, value)) = declaration.split_once(':') else {
                continue;
            };
            if name.trim() == "mask-type" {
                match value.trim() {
                    "alpha" => mask_type = MaskType::Alpha,
                    "luminance" => mask_type = MaskType::Luminance,
                    _ => {}
                }
            }
        }
    }
    Ok(mask_type)
}

pub(super) fn transform_segment(segment: &mut PathSegment, transform: Transform) {
    match segment {
        PathSegment::MoveTo(point) | PathSegment::LineTo(point) => {
            *point = transform.map(*point);
        }
        PathSegment::QuadraticTo { control, to } => {
            *control = transform.map(*control);
            *to = transform.map(*to);
        }
        PathSegment::CubicTo {
            control_0,
            control_1,
            to,
        } => {
            *control_0 = transform.map(*control_0);
            *control_1 = transform.map(*control_1);
            *to = transform.map(*to);
        }
        PathSegment::Close => {}
    }
}

pub(super) fn transform_scale(transform: Transform) -> f64 {
    transform
        .a
        .hypot(transform.c)
        .max(transform.b.hypot(transform.d))
}

pub(super) fn stroke_extent(style: &StrokeStyle, transform: Transform) -> f64 {
    let join = if style.line_join == LineJoin::Miter {
        style.miter_limit.max(1.0)
    } else {
        1.0
    };
    let cap = if style.line_cap == LineCap::Square {
        std::f64::consts::SQRT_2
    } else {
        1.0
    };
    style.width.max(0.0) * transform_scale(transform) * join.max(cap) / 2.0
}
