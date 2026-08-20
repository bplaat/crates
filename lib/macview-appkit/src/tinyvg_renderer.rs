/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::Arc;

use objc2::ffi::{objc_msgSendSuper, objc_super};
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};
use tinyvg::{Color, ColorSpace, Command, Document, Path, PathOperation, Point, Style};

use crate::cocoa::*;

struct TinyVgViewIvars {
    document: Cell<*mut Arc<Document>>,
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

            render_fitted(context, &*document, bounds.size, backing_scale);
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
/// The document is drawn to the edges of the view, which is what the scroll view of the viewer and
/// the panel of a preview both want. The caller owns the returned view and must send it `release`.
pub fn create_tinyvg_view(frame: Rect, document: Arc<Document>) -> *mut Object {
    // SAFETY: TinyVgView is a registered NSView subclass. The zero-initialized pointer ivar is
    // replaced with ownership of document before the view can draw.
    unsafe {
        let view: *mut Object = msg_send![TinyVgView::class(), alloc];
        let view: *mut Object = msg_send![view, initWithFrame: frame];
        assert!(!view.is_null(), "failed to create TinyVG view");
        let view_ref = &*(view.cast::<TinyVgView>());
        view_ref
            .ivars()
            .document
            .set(Box::into_raw(Box::new(document)));
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
        render_fitted(context, document, bounds, backing_scale);
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
) {
    if document.size.width <= 0.0 || document.size.height <= 0.0 {
        return;
    }
    let scale = (bounds.width.max(1.0) / document.size.width)
        .min(bounds.height.max(1.0) / document.size.height)
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

/// Returns the width to stroke `line_width` with, in the current user space.
///
/// Strokes thinner than a device pixel are passed through untouched: Core Graphics antialiases
/// them into a proportionally faint line, which keeps the drawing's balance at thumbnail sizes.
/// Only a degenerate width falls back to `hairline`, the user space size of one device pixel,
/// because Core Graphics draws a zero width stroke at full strength.
fn stroke_width(line_width: f64, hairline: f64) -> f64 {
    if line_width > 0.0 {
        line_width
    } else {
        hairline
    }
}

fn render(context: *mut c_void, document: &Document, hairline: f64) {
    for command in &document.commands {
        match command {
            Command::Fill { path, style } => {
                add_path(context, path);
                paint_path(context, style, Paint::Fill);
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
                        Paint::Stroke(stroke_width(*line_width, hairline)),
                    );
                } else {
                    stroke_variable_path(context, path, style, *line_width, hairline);
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

fn paint_path(context: *mut c_void, style: &Style, paint: Paint) {
    // SAFETY: context contains a current path built from finite parsed coordinates.
    unsafe {
        if let Paint::Stroke(width) = paint {
            CGContextSetLineWidth(context, width);
            CGContextSetLineCap(context, LINE_CAP_ROUND);
            CGContextSetLineJoin(context, LINE_JOIN_ROUND);
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
    hairline: f64,
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
                Paint::Stroke(stroke_width((width + next_width) / 2.0, hairline)),
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

    /// The side of the square test document, which renders one unit per pixel.
    const SIDE: usize = 32;

    fn solid(red: f64, green: f64, blue: f64) -> Style {
        Style::Solid(Color {
            red,
            green,
            blue,
            alpha: 1.0,
            color_space: ColorSpace::Srgb,
        })
    }

    fn path(start: Point, operations: impl IntoIterator<Item = PathOperation>) -> Path {
        Path {
            subpaths: vec![tinyvg::Subpath {
                start,
                nodes: operations
                    .into_iter()
                    .map(|operation| tinyvg::PathNode {
                        operation,
                        line_width: None,
                    })
                    .collect(),
            }],
        }
    }

    /// Renders `commands` over a white background and returns the `SIDE` square RGBA pixels.
    fn render_to_bitmap(commands: Vec<Command>) -> Vec<u8> {
        let mut pixels = vec![0u8; SIDE * SIDE * 4];
        let size = Size {
            width: SIDE as f64,
            height: SIDE as f64,
        };
        // SAFETY: The pixel buffer outlives the bitmap context and the supplied format is 8-bit
        // premultiplied RGBA. Created Core Graphics objects are released below.
        unsafe {
            let color_space = CGColorSpaceCreateDeviceRGB();
            assert!(!color_space.is_null());
            let context = CGBitmapContextCreate(
                pixels.as_mut_ptr().cast::<c_void>(),
                SIDE,
                SIDE,
                8,
                SIDE * 4,
                color_space,
                1,
            );
            CGColorSpaceRelease(color_space);
            assert!(!context.is_null());
            let document = Document {
                size: tinyvg::Size {
                    width: size.width,
                    height: size.height,
                },
                commands,
            };
            fill_white_background(context, size);
            render_tinyvg(context, &document, size, 1.0);
            CGContextRelease(context);
        }
        pixels
    }

    fn pixel(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
        let offset = (y * SIDE + x) * 4;
        pixels[offset..offset + 4].try_into().expect("RGBA pixel")
    }

    #[test]
    fn renders_a_filled_path_into_a_bitmap_context() {
        let pixels = render_to_bitmap(vec![Command::Fill {
            path: path(
                Point { x: 8.0, y: 8.0 },
                [
                    PathOperation::LineTo(Point { x: 24.0, y: 8.0 }),
                    PathOperation::LineTo(Point { x: 24.0, y: 24.0 }),
                    PathOperation::LineTo(Point { x: 8.0, y: 24.0 }),
                    PathOperation::Close,
                ],
            ),
            style: solid(1.0, 0.0, 0.0),
        }]);
        assert_eq!(pixel(&pixels, SIDE / 2, SIDE / 2), [255, 0, 0, 255]);
        assert_eq!(pixel(&pixels, 0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn stroke_width_only_replaces_a_degenerate_width() {
        // Anything positive is passed through so Core Graphics can fade sub-pixel strokes
        // instead of promoting them to a full strength hairline.
        assert_eq!(stroke_width(0.04, 1.0), 0.04);
        assert_eq!(stroke_width(3.0, 1.0), 3.0);
        assert_eq!(stroke_width(0.0, 0.25), 0.25);
        assert_eq!(stroke_width(-1.0, 0.25), 0.25);
    }

    #[test]
    fn sub_pixel_strokes_render_faintly_instead_of_solid_black() {
        let darkest = |line_width| {
            let middle = SIDE as f64 / 2.0;
            let pixels = render_to_bitmap(vec![Command::Stroke {
                path: path(
                    Point { x: middle, y: 0.0 },
                    [PathOperation::LineTo(Point {
                        x: middle,
                        y: SIDE as f64,
                    })],
                ),
                style: solid(0.0, 0.0, 0.0),
                line_width,
            }]);
            (0..SIDE)
                .map(|x| pixel(&pixels, x, SIDE / 2)[0])
                .min()
                .expect("row has pixels")
        };

        // A hair thin stroke barely tints the background, while a wide one is solid black.
        assert!(darkest(0.05) > 220);
        assert!(darkest(0.5) > 128);
        assert_eq!(darkest(4.0), 0);
    }
}
