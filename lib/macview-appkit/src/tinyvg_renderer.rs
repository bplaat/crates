/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;

use objc2::ffi::{objc_msgSendSuper, objc_super};
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};
use tinyvg::{Color, ColorSpace, Command, Document, Path, PathOperation, Point, Style};

use crate::cocoa::*;

struct TinyVgViewIvars {
    document: Cell<*mut Document>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "MacViewTinyVgView"]
    #[ivars = TinyVgViewIvars]
    struct TinyVgView;

    impl TinyVgView {
        #[unsafe(method(drawRect:))]
        fn _draw_rect(&self, _: Rect) {
            self.draw();
        }

        #[unsafe(method(isFlipped))]
        const fn _is_flipped(&self) -> Bool {
            Bool::YES
        }

        #[unsafe(method(isOpaque))]
        const fn _is_opaque(&self) -> Bool {
            Bool::NO
        }

        #[unsafe(method(dealloc))]
        fn _dealloc(&self) {
            self.dealloc();
        }
    }
);

impl TinyVgView {
    fn draw(&self) {
        let document = self.ivars().document.get();
        if document.is_null() {
            return;
        }
        // SAFETY: The view owns document until dealloc and drawing happens on AppKit's main
        // thread. NSGraphicsContext supplies a valid CGContext for the active draw pass.
        unsafe {
            let graphics_context: *mut Object =
                msg_send![class!(NSGraphicsContext), currentContext];
            if graphics_context.is_null() {
                return;
            }
            let context: *mut c_void = msg_send![graphics_context, CGContext];
            if context.is_null() {
                return;
            }
            let this = self as *const Self as *mut Object;
            let bounds: Rect = msg_send![this, bounds];

            let window: *mut Object = msg_send![this, window];
            let backing_scale: f64 = if window.is_null() {
                1.0
            } else {
                msg_send![window, backingScaleFactor]
            };

            render_fitted(context, &*document, bounds.size, backing_scale, 16.0);
        }
    }

    fn dealloc(&self) {
        let document = self.ivars().document.replace(std::ptr::null_mut());
        // SAFETY: The ivar owns document when non-null. objc_msgSendSuper invokes NSView's
        // dealloc with the exact Objective-C deallocation ABI.
        unsafe {
            if !document.is_null() {
                drop(Box::from_raw(document));
            }
            let super_info = objc_super {
                receiver: self as *const Self as *mut Object,
                super_class: class!(NSView).cast::<AnyClass>(),
            };
            let send: unsafe extern "C" fn(*const objc_super, *const c_void) =
                std::mem::transmute(objc_msgSendSuper as *const c_void);
            send(&super_info, sel!(dealloc).0);
        }
    }
}

/// Creates an owned, resize-aware `NSView` that displays a TinyVG document.
///
/// The caller owns the returned view and must send it `release`.
pub fn create_tinyvg_view(frame: Rect, document: Box<Document>) -> *mut Object {
    // SAFETY: TinyVgView is a registered NSView subclass. The zero-initialized pointer ivar is
    // replaced with ownership of document before the view can draw.
    unsafe {
        let view: *mut Object = msg_send![TinyVgView::class(), alloc];
        let view: *mut Object = msg_send![view, initWithFrame: frame];
        assert!(!view.is_null(), "failed to create TinyVG view");
        let view_ref = &*(view.cast::<TinyVgView>());
        view_ref.ivars().document.set(Box::into_raw(document));
        view
    }
}

/// Renders a TinyVG document aspect-fitted into a Core Graphics context.
///
/// TinyVG uses a top-left origin, so this converts the default bottom-left Core Graphics
/// coordinate system used by thumbnail contexts. The context is left in its original graphics
/// state. Unlike the view, this function does not paint a background, which preserves document
/// transparency for thumbnails.
///
/// # Safety
///
/// `context` must be a valid `CGContext` for the duration of the call.
pub unsafe fn render_tinyvg(
    context: *mut c_void,
    document: &Document,
    bounds: Size,
    backing_scale: f64,
) {
    // SAFETY: The caller guarantees context is valid.
    unsafe {
        CGContextSaveGState(context);
        CGContextTranslateCTM(context, 0.0, bounds.height);
        CGContextScaleCTM(context, 1.0, -1.0);
        render_fitted(context, document, bounds, backing_scale, 0.0);
        CGContextRestoreGState(context);
    }
}

/// Fills a Core Graphics context with an opaque white thumbnail background.
///
/// # Safety
///
/// `context` must be a valid `CGContext` for the duration of the call.
pub unsafe fn fill_white_background(context: *mut c_void, bounds: Size) {
    // SAFETY: The caller guarantees context is valid and bounds contains finite dimensions.
    unsafe {
        CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0);
        CGContextFillRect(
            context,
            Rect {
                origin: CgPoint { x: 0.0, y: 0.0 },
                size: bounds,
            },
        );
    }
}

unsafe fn render_fitted(
    context: *mut c_void,
    document: &Document,
    bounds: Size,
    backing_scale: f64,
    margin: f64,
) {
    if document.size.width <= 0.0 || document.size.height <= 0.0 {
        return;
    }
    let available_width = (bounds.width - margin * 2.0).max(1.0);
    let available_height = (bounds.height - margin * 2.0).max(1.0);
    let scale = (available_width / document.size.width)
        .min(available_height / document.size.height)
        .max(f64::EPSILON);
    let origin_x = (bounds.width - document.size.width * scale) / 2.0;
    let origin_y = (bounds.height - document.size.height * scale) / 2.0;
    // SAFETY: context is valid and all transformations are finite.
    unsafe {
        CGContextSaveGState(context);
        CGContextTranslateCTM(context, origin_x, origin_y);
        CGContextScaleCTM(context, scale, scale);
        render(context, document, 1.0 / (scale * backing_scale.max(1.0)));
        CGContextRestoreGState(context);
    }
}

fn render(context: *mut c_void, document: &Document, minimum_line_width: f64) {
    for command in &document.commands {
        match command {
            Command::Fill { path, style } => {
                add_path(context, path);
                paint_path(context, style, Paint::Fill, minimum_line_width);
            }
            Command::Stroke {
                path,
                style,
                line_width,
            } => {
                if path
                    .subpaths
                    .iter()
                    .all(|subpath| subpath.nodes.iter().all(|node| node.line_width.is_none()))
                {
                    add_path(context, path);
                    paint_path(
                        context,
                        style,
                        Paint::Stroke(line_width.max(minimum_line_width)),
                        minimum_line_width,
                    );
                } else {
                    stroke_variable_path(context, path, style, *line_width, minimum_line_width);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Paint {
    Fill,
    Stroke(f64),
}

fn paint_path(context: *mut c_void, style: &Style, paint: Paint, minimum_line_width: f64) {
    // SAFETY: context contains a current path built from finite parsed coordinates.
    unsafe {
        match paint {
            Paint::Fill => {}
            Paint::Stroke(width) => {
                CGContextSetLineWidth(context, width.max(minimum_line_width));
                CGContextSetLineCap(context, LINE_CAP_ROUND);
                CGContextSetLineJoin(context, LINE_JOIN_ROUND);
            }
        }
        if let Style::Solid(color) = style {
            let color = display_color(*color);
            match paint {
                Paint::Fill => CGContextSetRGBFillColor(
                    context,
                    color.red,
                    color.green,
                    color.blue,
                    color.alpha,
                ),
                Paint::Stroke(_) => CGContextSetRGBStrokeColor(
                    context,
                    color.red,
                    color.green,
                    color.blue,
                    color.alpha,
                ),
            }
            CGContextDrawPath(
                context,
                match paint {
                    Paint::Fill => DRAW_PATH_EVEN_ODD_FILL,
                    Paint::Stroke(_) => DRAW_PATH_STROKE,
                },
            );
            return;
        }

        CGContextSaveGState(context);
        match paint {
            Paint::Fill => CGContextEOClip(context),
            Paint::Stroke(_) => {
                CGContextReplacePathWithStrokedPath(context);
                CGContextClip(context);
            }
        }
        draw_gradient(context, style);
        CGContextRestoreGState(context);
    }
}

fn display_color(mut color: Color) -> Color {
    if color.color_space == ColorSpace::LinearSrgb {
        color.red = color.red.max(0.0).powf(1.0 / 2.2);
        color.green = color.green.max(0.0).powf(1.0 / 2.2);
        color.blue = color.blue.max(0.0).powf(1.0 / 2.2);
    }
    color
}

fn linear_components(color: Color) -> [f64; 4] {
    let convert = |value: f64| match color.color_space {
        ColorSpace::Srgb => value.max(0.0).powf(2.2),
        ColorSpace::LinearSrgb => value,
    };
    [
        convert(color.red),
        convert(color.green),
        convert(color.blue),
        color.alpha,
    ]
}

fn draw_gradient(context: *mut c_void, style: &Style) {
    let (start, end, start_color, end_color, radial) = match style {
        Style::LinearGradient {
            start,
            end,
            start_color,
            end_color,
        } => (*start, *end, *start_color, *end_color, false),
        Style::RadialGradient {
            center,
            edge,
            center_color,
            edge_color,
        } => (*center, *edge, *center_color, *edge_color, true),
        Style::Solid(_) => return,
    };
    let first = linear_components(start_color);
    let second = linear_components(end_color);
    let components = [
        first[0], first[1], first[2], first[3], second[0], second[1], second[2], second[3],
    ];
    let locations = [0.0, 1.0];
    // SAFETY: Core Graphics copies the component and location arrays. Every created object is
    // checked and released after the synchronous draw.
    unsafe {
        let color_space = CGColorSpaceCreateWithName(kCGColorSpaceLinearSRGB);
        if color_space.is_null() {
            return;
        }
        let gradient = CGGradientCreateWithColorComponents(
            color_space,
            components.as_ptr(),
            locations.as_ptr(),
            2,
        );
        CGColorSpaceRelease(color_space);
        if gradient.is_null() {
            return;
        }
        let options = GRADIENT_DRAWS_BEFORE_START | GRADIENT_DRAWS_AFTER_END;
        if radial {
            let radius = (end.x - start.x).hypot(end.y - start.y);
            CGContextDrawRadialGradient(context, gradient, start, 0.0, start, radius, options);
        } else {
            CGContextDrawLinearGradient(context, gradient, start, end, options);
        }
        CGGradientRelease(gradient);
    }
}

fn add_path(context: *mut c_void, path: &Path) {
    // SAFETY: context is valid and parsed points contain finite coordinate values.
    unsafe {
        CGContextBeginPath(context);
    }
    for subpath in &path.subpaths {
        // SAFETY: context is valid and the point is finite.
        unsafe {
            CGContextMoveToPoint(context, subpath.start.x, subpath.start.y);
        }
        let mut current = subpath.start;
        for node in &subpath.nodes {
            current = add_operation(context, current, subpath.start, &node.operation);
        }
    }
}

fn add_operation(
    context: *mut c_void,
    current: Point,
    start: Point,
    operation: &PathOperation,
) -> Point {
    // SAFETY: context is valid and the parser guarantees finite coordinates and radii.
    unsafe {
        match operation {
            PathOperation::LineTo(to) => CGContextAddLineToPoint(context, to.x, to.y),
            PathOperation::CubicTo {
                control_0,
                control_1,
                to,
            } => CGContextAddCurveToPoint(
                context,
                control_0.x,
                control_0.y,
                control_1.x,
                control_1.y,
                to.x,
                to.y,
            ),
            PathOperation::QuadraticTo { control, to } => {
                CGContextAddQuadCurveToPoint(context, control.x, control.y, to.x, to.y);
            }
            PathOperation::ArcTo {
                radius_x,
                radius_y,
                rotation,
                large_arc,
                sweep,
                to,
            } => {
                for curve in arc_curves(
                    current, *to, *radius_x, *radius_y, *rotation, *large_arc, *sweep,
                ) {
                    CGContextAddCurveToPoint(
                        context,
                        curve.control_0.x,
                        curve.control_0.y,
                        curve.control_1.x,
                        curve.control_1.y,
                        curve.to.x,
                        curve.to.y,
                    );
                }
            }
            PathOperation::Close => CGContextClosePath(context),
        }
    }
    match operation {
        PathOperation::LineTo(to)
        | PathOperation::CubicTo { to, .. }
        | PathOperation::QuadraticTo { to, .. }
        | PathOperation::ArcTo { to, .. } => *to,
        PathOperation::Close => start,
    }
}

fn stroke_variable_path(
    context: *mut c_void,
    path: &Path,
    style: &Style,
    initial_width: f64,
    minimum_line_width: f64,
) {
    for subpath in &path.subpaths {
        let mut current = subpath.start;
        let mut width = initial_width;
        for node in &subpath.nodes {
            let next_width = node.line_width.unwrap_or(width);
            // SAFETY: context is valid and current is a finite parsed point.
            unsafe {
                CGContextBeginPath(context);
                CGContextMoveToPoint(context, current.x, current.y);
            }
            let end = if matches!(node.operation, PathOperation::Close) {
                // A variable-width span starts a fresh Core Graphics subpath, so close it with
                // an explicit line to the TinyVG subpath origin.
                // SAFETY: context is valid and the subpath start is a finite parsed point.
                unsafe {
                    CGContextAddLineToPoint(context, subpath.start.x, subpath.start.y);
                }
                subpath.start
            } else {
                add_operation(context, current, subpath.start, &node.operation)
            };
            paint_path(
                context,
                style,
                Paint::Stroke(((width + next_width) / 2.0).max(minimum_line_width)),
                minimum_line_width,
            );
            current = end;
            width = next_width;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CubicCurve {
    control_0: Point,
    control_1: Point,
    to: Point,
}

fn arc_curves(
    from: Point,
    to: Point,
    mut radius_x: f64,
    mut radius_y: f64,
    rotation: f64,
    large_arc: bool,
    tinyvg_sweep: bool,
) -> Vec<CubicCurve> {
    if from == to {
        return Vec::new();
    }
    if radius_x == 0.0 || radius_y == 0.0 {
        let delta = Point {
            x: to.x - from.x,
            y: to.y - from.y,
        };
        return vec![CubicCurve {
            control_0: Point {
                x: from.x + delta.x / 3.0,
                y: from.y + delta.y / 3.0,
            },
            control_1: Point {
                x: from.x + delta.x * 2.0 / 3.0,
                y: from.y + delta.y * 2.0 / 3.0,
            },
            to,
        }];
    }

    radius_x = radius_x.abs();
    radius_y = radius_y.abs();
    let angle = rotation.to_radians();
    let (sin_angle, cos_angle) = angle.sin_cos();
    let half_dx = (from.x - to.x) / 2.0;
    let half_dy = (from.y - to.y) / 2.0;
    let transformed_x = cos_angle * half_dx + sin_angle * half_dy;
    let transformed_y = -sin_angle * half_dx + cos_angle * half_dy;
    let radii_scale =
        transformed_x.powi(2) / radius_x.powi(2) + transformed_y.powi(2) / radius_y.powi(2);
    if radii_scale > 1.0 {
        let scale = radii_scale.sqrt();
        radius_x *= scale;
        radius_y *= scale;
    }

    // TinyVG defines sweep as a left turn; SVG's endpoint formula uses the opposite flag in
    // the same top-left-origin coordinate system.
    let sweep = !tinyvg_sweep;
    let numerator = (radius_x * radius_y).powi(2)
        - (radius_x * transformed_y).powi(2)
        - (radius_y * transformed_x).powi(2);
    let denominator = (radius_x * transformed_y).powi(2) + (radius_y * transformed_x).powi(2);
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let factor = if denominator == 0.0 {
        0.0
    } else {
        sign * (numerator.max(0.0) / denominator).sqrt()
    };
    let center_x_transformed = factor * radius_x * transformed_y / radius_y;
    let center_y_transformed = factor * -radius_y * transformed_x / radius_x;
    let center = Point {
        x: cos_angle * center_x_transformed - sin_angle * center_y_transformed
            + (from.x + to.x) / 2.0,
        y: sin_angle * center_x_transformed
            + cos_angle * center_y_transformed
            + (from.y + to.y) / 2.0,
    };

    let vector_angle =
        |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let start_vector = (
        (transformed_x - center_x_transformed) / radius_x,
        (transformed_y - center_y_transformed) / radius_y,
    );
    let end_vector = (
        (-transformed_x - center_x_transformed) / radius_x,
        (-transformed_y - center_y_transformed) / radius_y,
    );
    let start_angle = vector_angle(1.0, 0.0, start_vector.0, start_vector.1);
    let mut delta_angle = vector_angle(start_vector.0, start_vector.1, end_vector.0, end_vector.1);
    if !sweep && delta_angle > 0.0 {
        delta_angle -= std::f64::consts::TAU;
    } else if sweep && delta_angle < 0.0 {
        delta_angle += std::f64::consts::TAU;
    }
    let segment_count = (delta_angle.abs() / std::f64::consts::FRAC_PI_2).ceil() as usize;
    let segment_angle = delta_angle / segment_count as f64;
    let mut curves = Vec::with_capacity(segment_count);
    for index in 0..segment_count {
        let first_angle = start_angle + segment_angle * index as f64;
        let second_angle = first_angle + segment_angle;
        let alpha = 4.0 / 3.0 * (segment_angle / 4.0).tan();
        let map = |x: f64, y: f64| Point {
            x: center.x + cos_angle * radius_x * x - sin_angle * radius_y * y,
            y: center.y + sin_angle * radius_x * x + cos_angle * radius_y * y,
        };
        let (first_sin, first_cos) = first_angle.sin_cos();
        let (second_sin, second_cos) = second_angle.sin_cos();
        curves.push(CubicCurve {
            control_0: map(first_cos - alpha * first_sin, first_sin + alpha * first_cos),
            control_1: map(
                second_cos + alpha * second_sin,
                second_sin - alpha * second_cos,
            ),
            to: if index + 1 == segment_count {
                to
            } else {
                map(second_cos, second_sin)
            },
        });
    }
    curves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_arc_is_split_into_finite_cubic_curves() {
        let curves = arc_curves(
            Point { x: 0.0, y: 0.0 },
            Point { x: 100.0, y: 0.0 },
            50.0,
            50.0,
            0.0,
            false,
            false,
        );
        assert_eq!(curves.len(), 2);
        assert_eq!(
            curves.last().expect("arc has curves").to,
            Point { x: 100.0, y: 0.0 }
        );
        assert!(curves.iter().all(|curve| curve.control_0.x.is_finite()
            && curve.control_0.y.is_finite()
            && curve.control_1.x.is_finite()
            && curve.control_1.y.is_finite()));
    }

    #[test]
    fn undersized_ellipse_radii_are_scaled_to_fit() {
        let curves = arc_curves(
            Point { x: 0.0, y: 0.0 },
            Point { x: 200.0, y: 0.0 },
            10.0,
            5.0,
            30.0,
            false,
            true,
        );
        assert!(!curves.is_empty());
        assert_eq!(curves.last().expect("arc has curves").to.x, 200.0);
    }

    #[test]
    fn renders_into_a_bitmap_context() {
        let mut pixels = [0u8; 16 * 16 * 4];
        // SAFETY: The mutable pixel buffer outlives the bitmap context and the supplied format
        // is 8-bit premultiplied RGBA. Created Core Graphics objects are released below.
        unsafe {
            let color_space = CGColorSpaceCreateDeviceRGB();
            assert!(!color_space.is_null());
            let context = CGBitmapContextCreate(
                pixels.as_mut_ptr().cast::<c_void>(),
                16,
                16,
                8,
                16 * 4,
                color_space,
                1,
            );
            CGColorSpaceRelease(color_space);
            assert!(!context.is_null());
            let document = Document {
                size: tinyvg::Size {
                    width: 16.0,
                    height: 16.0,
                },
                commands: vec![Command::Fill {
                    path: Path {
                        subpaths: vec![tinyvg::Subpath {
                            start: Point { x: 2.0, y: 2.0 },
                            nodes: vec![
                                tinyvg::PathNode {
                                    operation: PathOperation::LineTo(Point { x: 14.0, y: 2.0 }),
                                    line_width: None,
                                },
                                tinyvg::PathNode {
                                    operation: PathOperation::LineTo(Point { x: 14.0, y: 14.0 }),
                                    line_width: None,
                                },
                                tinyvg::PathNode {
                                    operation: PathOperation::LineTo(Point { x: 2.0, y: 14.0 }),
                                    line_width: None,
                                },
                                tinyvg::PathNode {
                                    operation: PathOperation::Close,
                                    line_width: None,
                                },
                            ],
                        }],
                    },
                    style: Style::Solid(Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                        color_space: ColorSpace::Srgb,
                    }),
                }],
            };
            fill_white_background(
                context,
                Size {
                    width: 16.0,
                    height: 16.0,
                },
            );
            render_tinyvg(
                context,
                &document,
                Size {
                    width: 16.0,
                    height: 16.0,
                },
                1.0,
            );
            CGContextRelease(context);
        }
        assert!(pixels.iter().any(|component| *component != 0));
    }
}
