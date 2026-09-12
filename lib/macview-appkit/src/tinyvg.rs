/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::null;

use ::tinyvg::{Color, ColorSpace, Command, Document, Path, PathOperation, Point, Style, Subpath};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

use crate::headers::*;

struct TinyVgImageRepIvars {
    document: Document,
    prepared_commands: Vec<PreparedCommand>,
}

struct PreparedCommand {
    bounds: Option<CommandBounds>,
    path: Option<CachedPath>,
    style: PreparedStyle,
    variable_width: bool,
}

impl PreparedCommand {
    fn new(command: &Command) -> Self {
        let (path, style, variable_width) = match command {
            Command::Fill { path, style } => (path, style, false),
            Command::Stroke { path, style, .. } => (
                path,
                style,
                path.subpaths
                    .iter()
                    .any(|subpath| subpath.nodes.iter().any(|node| node.line_width.is_some())),
            ),
        };
        Self {
            bounds: command_bounds(command),
            path: if variable_width {
                None
            } else {
                CachedPath::new(path)
            },
            style: PreparedStyle::new(style),
            variable_width,
        }
    }
}

enum PreparedStyle {
    Solid(Color),
    LinearGradient {
        gradient: Option<CachedGradient>,
        start: Point,
        end: Point,
    },
    RadialGradient {
        gradient: Option<CachedGradient>,
        center: Point,
        edge: Point,
    },
}

impl PreparedStyle {
    fn new(style: &Style) -> Self {
        match style {
            Style::Solid(color) => Self::Solid(display_color(*color)),
            Style::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            } => Self::LinearGradient {
                gradient: CachedGradient::new(*start_color, *end_color),
                start: *start,
                end: *end,
            },
            Style::RadialGradient {
                center,
                edge,
                center_color,
                edge_color,
            } => Self::RadialGradient {
                gradient: CachedGradient::new(*center_color, *edge_color),
                center: *center,
                edge: *edge,
            },
        }
    }
}

struct CachedGradient(*const c_void);

impl CachedGradient {
    fn new(start_color: Color, end_color: Color) -> Option<Self> {
        let first = linear_components(start_color);
        let second = linear_components(end_color);
        let components = [
            first[0], first[1], first[2], first[3], second[0], second[1], second[2], second[3],
        ];
        let locations = [0.0, 1.0];
        // SAFETY: Core Graphics copies the component and location arrays. The returned immutable
        // gradient owns its data and can be reused by later drawing contexts.
        unsafe {
            let color_space = CGColorSpaceCreateWithName(kCGColorSpaceLinearSRGB);
            if color_space.is_null() {
                return None;
            }
            let gradient = CGGradientCreateWithColorComponents(
                color_space,
                components.as_ptr(),
                locations.as_ptr(),
                2,
            );
            CGColorSpaceRelease(color_space);
            (!gradient.is_null()).then_some(Self(gradient))
        }
    }
}

impl Drop for CachedGradient {
    fn drop(&mut self) {
        // SAFETY: The gradient was returned with ownership by Core Graphics.
        unsafe { CGGradientRelease(self.0) }
    }
}

enum CachedPath {
    Whole(NativePath),
    Subpaths(Vec<CachedSubpath>),
}

impl CachedPath {
    fn new(path: &Path) -> Option<Self> {
        if path.subpaths.len() <= 1 {
            return NativePath::new(path).map(Self::Whole);
        }
        path.subpaths
            .iter()
            .map(|subpath| {
                Some(CachedSubpath {
                    bounds: subpath_bounds(subpath),
                    path: NativePath::new_subpath(subpath)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .map(Self::Subpaths)
    }

    fn add_to_context(
        &self,
        context: *mut c_void,
        clip: CommandBounds,
        hairline: f64,
        stroke_radius: f64,
    ) {
        // SAFETY: Every stored path is immutable and alive for this synchronous draw.
        unsafe {
            match self {
                Self::Whole(path) => CGContextAddPath(context, path.0),
                Self::Subpaths(subpaths) => {
                    for subpath in subpaths {
                        if subpath.bounds.intersects(clip, hairline + stroke_radius) {
                            CGContextAddPath(context, subpath.path.0);
                        }
                    }
                }
            }
        }
    }
}

struct CachedSubpath {
    bounds: CommandBounds,
    path: NativePath,
}

struct NativePath(*const c_void);

impl NativePath {
    fn new(path: &Path) -> Option<Self> {
        // SAFETY: Core Graphics owns the mutable path until it is released below. The immutable
        // copy owns all path data and can be reused by later drawing contexts.
        unsafe {
            let mutable_path = CGPathCreateMutable();
            if mutable_path.is_null() {
                return None;
            }
            append_path(PathTarget::Path(mutable_path), path);
            let path = CGPathCreateCopy(mutable_path);
            CGPathRelease(mutable_path);
            (!path.is_null()).then_some(Self(path))
        }
    }

    fn new_subpath(subpath: &Subpath) -> Option<Self> {
        // SAFETY: Core Graphics owns the mutable path until it is released below. The immutable
        // copy owns all path data and can be reused by later drawing contexts.
        unsafe {
            let mutable_path = CGPathCreateMutable();
            if mutable_path.is_null() {
                return None;
            }
            append_subpath(PathTarget::Path(mutable_path), subpath);
            let path = CGPathCreateCopy(mutable_path);
            CGPathRelease(mutable_path);
            (!path.is_null()).then_some(Self(path))
        }
    }
}

impl Drop for NativePath {
    fn drop(&mut self) {
        // SAFETY: The path was returned with ownership by CGPathCreateCopy.
        unsafe { CGPathRelease(self.0) }
    }
}

define_class!(
    #[unsafe(super(NSImageRep))]
    #[name = "MacViewTinyVgImageRep"]
    #[ivars = TinyVgImageRepIvars]
    struct TinyVgImageRep;

    impl TinyVgImageRep {
        #[unsafe(method(draw))]
        fn _draw(&self) -> Bool {
            self.draw_in(Rect {
                origin: CgPoint { x: 0.0, y: 0.0 },
                size: Size {
                    width: self.ivars().document.size.width,
                    height: self.ivars().document.size.height,
                },
            })
        }
    }
);

impl TinyVgImageRep {
    fn draw_in(&self, rect: Rect) -> Bool {
        let ivars = self.ivars();
        let document = &ivars.document;
        // SAFETY: The representation owns document and NSGraphicsContext supplies a valid
        // CGContext for the active draw pass.
        unsafe {
            let graphics_context: *mut Object =
                msg_send![class!(NSGraphicsContext), currentContext];
            if graphics_context.is_null() {
                return Bool::NO;
            }
            let context: *mut c_void = msg_send![graphics_context, CGContext];
            if context.is_null() {
                return Bool::NO;
            }
            let flipped: Bool = msg_send![graphics_context, isFlipped];
            let device = CGContextConvertSizeToDeviceSpace(
                context,
                Size {
                    width: 1.0,
                    height: 1.0,
                },
            );
            let backing_scale = device.width.abs().max(device.height.abs()).max(1.0);
            CGContextSaveGState(context);
            CGContextTranslateCTM(context, rect.origin.x, rect.origin.y);
            if flipped.as_bool() {
                render_fitted(
                    context,
                    document,
                    &ivars.prepared_commands,
                    rect.size,
                    backing_scale,
                );
            } else {
                render_tinyvg(
                    context,
                    document,
                    &ivars.prepared_commands,
                    rect.size,
                    backing_scale,
                );
            }
            CGContextRestoreGState(context);
            Bool::YES
        }
    }
}

/// Creates an owned `NSImage` containing a vector TinyVG representation.
pub(crate) fn create_tinyvg_image(document: Document) -> Retained<Object> {
    let size = Size {
        width: document.size.width,
        height: document.size.height,
    };
    let prepared_commands = document.commands.iter().map(PreparedCommand::new).collect();
    // SAFETY: Both classes are registered AppKit classes. The representation owns the parsed
    // document and its prepared bounds, and NSImage retains it after it is added.
    unsafe {
        let representation: Allocated<TinyVgImageRep> = msg_send![TinyVgImageRep::class(), alloc];
        let representation: Retained<TinyVgImageRep> = msg_send![
            super(representation.set_ivars(TinyVgImageRepIvars {
                document,
                prepared_commands,
            })),
            init
        ];
        let _: () = msg_send![&*representation, setSize: size];
        let _: () = msg_send![&*representation, setAlpha: Bool::YES];
        let _: () = msg_send![&*representation, setOpaque: Bool::NO];

        let image: Allocated<Object> = msg_send![class!(NSImage), alloc];
        let image: Retained<Object> = msg_send![image, initWithSize: size];
        // Keep the scroll view's visible clip all the way to this vector representation. A cached
        // raster of the whole image becomes both expensive and blurry at large magnifications.
        let _: () = msg_send![&*image, setCacheMode: NS_IMAGE_CACHE_NEVER];
        let _: () = msg_send![&*image, addRepresentation: representation.as_ptr()];
        image
    }
}

/// Renders a TinyVG document aspect-fitted into a Core Graphics context.
///
/// TinyVG uses a top-left origin, so this converts the default bottom-left Core Graphics
/// coordinate system used by thumbnail contexts. The context is left in its original graphics
/// state. It does not paint a background, which preserves document transparency.
///
/// # Safety
///
/// `context` must be a valid `CGContext` for the duration of the call.
unsafe fn render_tinyvg(
    context: *mut c_void,
    document: &Document,
    prepared_commands: &[PreparedCommand],
    bounds: Size,
    backing_scale: f64,
) {
    // SAFETY: The caller guarantees context is valid.
    unsafe {
        CGContextSaveGState(context);
        CGContextTranslateCTM(context, 0.0, bounds.height);
        CGContextScaleCTM(context, 1.0, -1.0);
        render_fitted(context, document, prepared_commands, bounds, backing_scale);
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
    prepared_commands: &[PreparedCommand],
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
        let clip = CommandBounds::from_rect(CGContextGetClipBoundingBox(context));
        render(
            context,
            document,
            prepared_commands,
            clip,
            1.0 / (scale * backing_scale.max(1.0)),
        );
        CGContextRestoreGState(context);
    }
}

/// A conservative command extent in TinyVG document coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CommandBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    stroke_radius: f64,
}

impl CommandBounds {
    const fn from_point(point: Point) -> Self {
        Self {
            min_x: point.x,
            min_y: point.y,
            max_x: point.x,
            max_y: point.y,
            stroke_radius: 0.0,
        }
    }

    fn from_rect(rect: Rect) -> Self {
        let other_x = rect.origin.x + rect.size.width;
        let other_y = rect.origin.y + rect.size.height;
        Self {
            min_x: rect.origin.x.min(other_x),
            min_y: rect.origin.y.min(other_y),
            max_x: rect.origin.x.max(other_x),
            max_y: rect.origin.y.max(other_y),
            stroke_radius: 0.0,
        }
    }

    const fn include(&mut self, point: Point) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_x = self.max_x.max(point.x);
        self.max_y = self.max_y.max(point.y);
    }

    fn intersects(self, clip: Self, hairline: f64) -> bool {
        // Include one device pixel for antialiasing around fills and stroke edges.
        let padding = self.stroke_radius + hairline;
        self.max_x + padding >= clip.min_x
            && self.min_x - padding <= clip.max_x
            && self.max_y + padding >= clip.min_y
            && self.min_y - padding <= clip.max_y
    }
}

fn command_bounds(command: &Command) -> Option<CommandBounds> {
    let (path, stroke_radius) = match command {
        Command::Fill { path, .. } => (path, 0.0),
        Command::Stroke {
            path, line_width, ..
        } => {
            let maximum_width = path
                .subpaths
                .iter()
                .flat_map(|subpath| &subpath.nodes)
                .filter_map(|node| node.line_width)
                .fold((*line_width).max(0.0), f64::max);
            (path, maximum_width / 2.0)
        }
    };
    path_bounds(path).map(|mut bounds| {
        bounds.stroke_radius = stroke_radius;
        bounds
    })
}

fn path_bounds(path: &Path) -> Option<CommandBounds> {
    let mut bounds: Option<CommandBounds> = None;
    for subpath in &path.subpaths {
        let subpath_bounds = subpath_bounds(subpath);
        if let Some(bounds) = &mut bounds {
            bounds.include(Point {
                x: subpath_bounds.min_x,
                y: subpath_bounds.min_y,
            });
            bounds.include(Point {
                x: subpath_bounds.max_x,
                y: subpath_bounds.max_y,
            });
        } else {
            bounds = Some(subpath_bounds);
        }
    }
    bounds
}

fn subpath_bounds(subpath: &Subpath) -> CommandBounds {
    let mut bounds = CommandBounds::from_point(subpath.start);
    let mut current = subpath.start;
    for node in &subpath.nodes {
        match &node.operation {
            PathOperation::LineTo(to) => bounds.include(*to),
            PathOperation::CubicTo {
                control_0,
                control_1,
                to,
            } => {
                bounds.include(*control_0);
                bounds.include(*control_1);
                bounds.include(*to);
            }
            PathOperation::QuadraticTo { control, to } => {
                bounds.include(*control);
                bounds.include(*to);
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
                    bounds.include(curve.control_0);
                    bounds.include(curve.control_1);
                    bounds.include(curve.to);
                }
            }
            PathOperation::Close => bounds.include(subpath.start),
        }
        current = match &node.operation {
            PathOperation::LineTo(to)
            | PathOperation::CubicTo { to, .. }
            | PathOperation::QuadraticTo { to, .. }
            | PathOperation::ArcTo { to, .. } => *to,
            PathOperation::Close => subpath.start,
        };
    }
    bounds
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

fn render(
    context: *mut c_void,
    document: &Document,
    prepared_commands: &[PreparedCommand],
    clip: CommandBounds,
    hairline: f64,
) {
    for (command, prepared) in document.commands.iter().zip(prepared_commands) {
        if !prepared
            .bounds
            .is_some_and(|bounds| bounds.intersects(clip, hairline))
        {
            continue;
        }
        match command {
            Command::Fill { path, .. } => {
                add_path(
                    context,
                    path,
                    prepared.path.as_ref(),
                    clip,
                    hairline,
                    prepared.bounds.map_or(0.0, |bounds| bounds.stroke_radius),
                );
                paint_path(context, &prepared.style, Paint::Fill);
            }
            Command::Stroke {
                path, line_width, ..
            } => {
                if !prepared.variable_width {
                    add_path(
                        context,
                        path,
                        prepared.path.as_ref(),
                        clip,
                        hairline,
                        prepared.bounds.map_or(0.0, |bounds| bounds.stroke_radius),
                    );
                    paint_path(
                        context,
                        &prepared.style,
                        Paint::Stroke(stroke_width(*line_width, hairline)),
                    );
                } else {
                    stroke_variable_path(context, path, &prepared.style, *line_width, hairline);
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

fn paint_path(context: *mut c_void, style: &PreparedStyle, paint: Paint) {
    // SAFETY: context contains a current path built from finite parsed coordinates.
    unsafe {
        if let Paint::Stroke(width) = paint {
            CGContextSetLineWidth(context, width);
            CGContextSetLineCap(context, LINE_CAP_ROUND);
            CGContextSetLineJoin(context, LINE_JOIN_ROUND);
        }
        if let PreparedStyle::Solid(color) = style {
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

fn draw_gradient(context: *mut c_void, style: &PreparedStyle) {
    let (gradient, start, end, radial) = match style {
        PreparedStyle::LinearGradient {
            gradient,
            start,
            end,
        } => (gradient.as_ref(), *start, *end, false),
        PreparedStyle::RadialGradient {
            gradient,
            center,
            edge,
        } => (gradient.as_ref(), *center, *edge, true),
        PreparedStyle::Solid(_) => return,
    };
    let Some(gradient) = gradient else {
        return;
    };
    // SAFETY: The prepared immutable gradient remains alive for the synchronous draw.
    unsafe {
        let options = GRADIENT_DRAWS_BEFORE_START | GRADIENT_DRAWS_AFTER_END;
        if radial {
            let radius = (end.x - start.x).hypot(end.y - start.y);
            CGContextDrawRadialGradient(context, gradient.0, start, 0.0, start, radius, options);
        } else {
            CGContextDrawLinearGradient(context, gradient.0, start, end, options);
        }
    }
}

fn add_path(
    context: *mut c_void,
    path: &Path,
    cached_path: Option<&CachedPath>,
    clip: CommandBounds,
    hairline: f64,
    stroke_radius: f64,
) {
    // SAFETY: context is valid and parsed points contain finite coordinate values.
    unsafe {
        CGContextBeginPath(context);
        if let Some(cached_path) = cached_path {
            cached_path.add_to_context(context, clip, hairline, stroke_radius);
            return;
        }
    }
    append_path(PathTarget::Context(context), path);
}

#[derive(Clone, Copy)]
enum PathTarget {
    Context(*mut c_void),
    Path(*mut c_void),
}

impl PathTarget {
    unsafe fn move_to(self, point: Point) {
        // SAFETY: The caller guarantees that the target is valid and the point is finite.
        unsafe {
            match self {
                Self::Context(context) => CGContextMoveToPoint(context, point.x, point.y),
                Self::Path(path) => CGPathMoveToPoint(path, null(), point.x, point.y),
            }
        }
    }

    unsafe fn line_to(self, point: Point) {
        // SAFETY: The caller guarantees that the target is valid and the point is finite.
        unsafe {
            match self {
                Self::Context(context) => CGContextAddLineToPoint(context, point.x, point.y),
                Self::Path(path) => CGPathAddLineToPoint(path, null(), point.x, point.y),
            }
        }
    }

    unsafe fn cubic_to(self, control_0: Point, control_1: Point, to: Point) {
        // SAFETY: The caller guarantees that the target is valid and the points are finite.
        unsafe {
            match self {
                Self::Context(context) => CGContextAddCurveToPoint(
                    context,
                    control_0.x,
                    control_0.y,
                    control_1.x,
                    control_1.y,
                    to.x,
                    to.y,
                ),
                Self::Path(path) => CGPathAddCurveToPoint(
                    path,
                    null(),
                    control_0.x,
                    control_0.y,
                    control_1.x,
                    control_1.y,
                    to.x,
                    to.y,
                ),
            }
        }
    }

    unsafe fn quadratic_to(self, control: Point, to: Point) {
        // SAFETY: The caller guarantees that the target is valid and the points are finite.
        unsafe {
            match self {
                Self::Context(context) => {
                    CGContextAddQuadCurveToPoint(context, control.x, control.y, to.x, to.y);
                }
                Self::Path(path) => {
                    CGPathAddQuadCurveToPoint(path, null(), control.x, control.y, to.x, to.y);
                }
            }
        }
    }

    unsafe fn close(self) {
        // SAFETY: The caller guarantees that the target is valid.
        unsafe {
            match self {
                Self::Context(context) => CGContextClosePath(context),
                Self::Path(path) => CGPathCloseSubpath(path),
            }
        }
    }
}

fn append_path(target: PathTarget, path: &Path) {
    for subpath in &path.subpaths {
        append_subpath(target, subpath);
    }
}

fn append_subpath(target: PathTarget, subpath: &Subpath) {
    // SAFETY: target is valid and the point is finite.
    unsafe {
        target.move_to(subpath.start);
    }
    let mut current = subpath.start;
    for node in &subpath.nodes {
        current = add_operation(target, current, subpath.start, &node.operation);
    }
}

fn add_operation(
    target: PathTarget,
    current: Point,
    start: Point,
    operation: &PathOperation,
) -> Point {
    // SAFETY: target is valid and the parser guarantees finite coordinates and radii.
    unsafe {
        match operation {
            PathOperation::LineTo(to) => target.line_to(*to),
            PathOperation::CubicTo {
                control_0,
                control_1,
                to,
            } => target.cubic_to(*control_0, *control_1, *to),
            PathOperation::QuadraticTo { control, to } => {
                target.quadratic_to(*control, *to);
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
                    target.cubic_to(curve.control_0, curve.control_1, curve.to);
                }
            }
            PathOperation::Close => target.close(),
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
    style: &PreparedStyle,
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
                add_operation(
                    PathTarget::Context(context),
                    current,
                    subpath.start,
                    &node.operation,
                )
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
    use objc2::rc::autoreleasepool;

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
        Style::Solid(color(red, green, blue))
    }

    fn color(red: f64, green: f64, blue: f64) -> Color {
        Color {
            red,
            green,
            blue,
            alpha: 1.0,
            color_space: ColorSpace::Srgb,
        }
    }

    fn path(start: Point, operations: impl IntoIterator<Item = PathOperation>) -> Path {
        Path {
            subpaths: vec![::tinyvg::Subpath {
                start,
                nodes: operations
                    .into_iter()
                    .map(|operation| ::tinyvg::PathNode {
                        operation,
                        line_width: None,
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn command_bounds_cull_paths_outside_the_visible_clip() {
        let command = Command::Fill {
            path: path(
                Point { x: 100.0, y: 100.0 },
                [PathOperation::LineTo(Point { x: 120.0, y: 120.0 })],
            ),
            style: solid(1.0, 0.0, 0.0),
        };
        let bounds = command_bounds(&command).expect("command has geometry");
        let visible = CommandBounds::from_rect(Rect {
            origin: CgPoint { x: 0.0, y: 0.0 },
            size: Size {
                width: 50.0,
                height: 50.0,
            },
        });
        assert!(!bounds.intersects(visible, 1.0));

        let overlapping = CommandBounds::from_rect(Rect {
            origin: CgPoint { x: 110.0, y: 110.0 },
            size: Size {
                width: 50.0,
                height: 50.0,
            },
        });
        assert!(bounds.intersects(overlapping, 1.0));
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
                size: ::tinyvg::Size {
                    width: size.width,
                    height: size.height,
                },
                commands,
            };
            let prepared_commands = document
                .commands
                .iter()
                .map(PreparedCommand::new)
                .collect::<Vec<_>>();
            fill_white_background(context, size);
            render_tinyvg(context, &document, &prepared_commands, size, 1.0);
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
    fn renders_a_prepared_gradient_into_a_bitmap_context() {
        let pixels = render_to_bitmap(vec![Command::Fill {
            path: path(
                Point { x: 0.0, y: 0.0 },
                [
                    PathOperation::LineTo(Point {
                        x: SIDE as f64,
                        y: 0.0,
                    }),
                    PathOperation::LineTo(Point {
                        x: SIDE as f64,
                        y: SIDE as f64,
                    }),
                    PathOperation::LineTo(Point {
                        x: 0.0,
                        y: SIDE as f64,
                    }),
                    PathOperation::Close,
                ],
            ),
            style: Style::LinearGradient {
                start: Point { x: 0.0, y: 0.0 },
                end: Point {
                    x: SIDE as f64,
                    y: 0.0,
                },
                start_color: color(1.0, 0.0, 0.0),
                end_color: color(0.0, 0.0, 1.0),
            },
        }]);
        let left = pixel(&pixels, 2, SIDE / 2);
        let right = pixel(&pixels, SIDE - 3, SIDE / 2);
        assert!(left[0] > left[2]);
        assert!(right[2] > right[0]);
    }

    #[test]
    fn tinyvg_image_rep_draws_through_nsimage() {
        autoreleasepool(|_| {
            let mut pixels = vec![0u8; SIDE * SIDE * 4];
            let size = Size {
                width: SIDE as f64,
                height: SIDE as f64,
            };
            let document = Document {
                size: ::tinyvg::Size {
                    width: size.width / 2.0,
                    height: size.height / 2.0,
                },
                commands: vec![Command::Fill {
                    path: path(
                        Point { x: 4.0, y: 4.0 },
                        [
                            PathOperation::LineTo(Point { x: 12.0, y: 4.0 }),
                            PathOperation::LineTo(Point { x: 12.0, y: 12.0 }),
                            PathOperation::LineTo(Point { x: 4.0, y: 12.0 }),
                            PathOperation::Close,
                        ],
                    ),
                    style: solid(1.0, 0.0, 0.0),
                }],
            };
            // SAFETY: The pixel buffer outlives the bitmap context, the NSGraphicsContext wraps
            // that context synchronously, and all created Core Graphics objects are released.
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
                fill_white_background(context, size);

                let graphics_context: *mut Object = msg_send![class!(NSGraphicsContext),
                    graphicsContextWithCGContext: context,
                    flipped: Bool::NO
                ];
                let _: () = msg_send![class!(NSGraphicsContext), saveGraphicsState];
                let _: () =
                    msg_send![class!(NSGraphicsContext), setCurrentContext: graphics_context];
                let image = create_tinyvg_image(document);
                let _: () = msg_send![&*image,
                    drawInRect: Rect {
                        origin: CgPoint { x: 0.0, y: 0.0 },
                        size,
                    }
                ];
                let _: () = msg_send![class!(NSGraphicsContext), restoreGraphicsState];
                CGContextRelease(context);
            }
            assert_eq!(pixel(&pixels, SIDE / 2, SIDE / 2), [255, 0, 0, 255]);
            assert_eq!(pixel(&pixels, 0, 0), [255, 255, 255, 255]);
        });
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
