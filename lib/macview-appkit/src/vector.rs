/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::null;

use image::{
    Color, DrawCommand, FillRule, LineCap, LineJoin, MaskType, Paint, PathSegment, SpreadMethod,
    Transform, VectorColorSpace, VectorImage,
};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

use crate::headers::*;

struct VectorImageRepIvars {
    document: VectorImage,
    paths: Vec<NativePath>,
    paints: Vec<PreparedPaint>,
}
struct NativePath(*const c_void);

impl NativePath {
    fn new(segments: &[PathSegment]) -> Option<Self> {
        // SAFETY: The immutable copy owns all data appended to the temporary path.
        unsafe {
            let path = CGPathCreateMutable();
            if path.is_null() {
                return None;
            }
            for segment in segments {
                match *segment {
                    PathSegment::MoveTo(p) => CGPathMoveToPoint(path, null(), p.x, p.y),
                    PathSegment::LineTo(p) => CGPathAddLineToPoint(path, null(), p.x, p.y),
                    PathSegment::QuadraticTo { control, to } => {
                        CGPathAddQuadCurveToPoint(path, null(), control.x, control.y, to.x, to.y)
                    }
                    PathSegment::CubicTo {
                        control_0,
                        control_1,
                        to,
                    } => CGPathAddCurveToPoint(
                        path,
                        null(),
                        control_0.x,
                        control_0.y,
                        control_1.x,
                        control_1.y,
                        to.x,
                        to.y,
                    ),
                    PathSegment::Close => CGPathCloseSubpath(path),
                }
            }
            let copy = CGPathCreateCopy(path);
            CGPathRelease(path);
            (!copy.is_null()).then_some(Self(copy))
        }
    }
}
impl Drop for NativePath {
    fn drop(&mut self) {
        // SAFETY: This path was returned with ownership by Core Graphics.
        unsafe { CGPathRelease(self.0) }
    }
}

enum PreparedPaint {
    Solid(Color),
    Linear {
        gradients: GradientSet,
        start: image::Point,
        end: image::Point,
        transform: Transform,
        spread: SpreadMethod,
    },
    Radial {
        gradients: GradientSet,
        center: image::Point,
        focal: image::Point,
        radius: f64,
        transform: Transform,
        spread: SpreadMethod,
    },
}
impl PreparedPaint {
    fn new(paint: &Paint, modes: u8) -> Self {
        match paint {
            Paint::Solid(color) => Self::Solid(display_color(*color)),
            Paint::LinearGradient {
                start,
                end,
                stops,
                spread,
                transform,
            } => Self::Linear {
                gradients: GradientSet::new(stops, *spread, modes),
                start: *start,
                end: *end,
                transform: *transform,
                spread: *spread,
            },
            Paint::RadialGradient {
                center,
                focal,
                radius,
                stops,
                spread,
                transform,
            } => Self::Radial {
                gradients: GradientSet::new(stops, *spread, modes),
                center: *center,
                focal: *focal,
                radius: *radius,
                transform: *transform,
                spread: *spread,
            },
        }
    }
}
struct GradientSet {
    normal: Option<NativeGradient>,
    normal_reverse: Option<NativeGradient>,
    alpha: Option<NativeGradient>,
    alpha_reverse: Option<NativeGradient>,
    luminance: Option<NativeGradient>,
    luminance_reverse: Option<NativeGradient>,
}
impl GradientSet {
    fn new(stops: &[image::GradientStop], spread: SpreadMethod, modes: u8) -> Self {
        let has = |mode: RenderMode| modes & mode.bit() != 0;
        let reversed = spread == SpreadMethod::Reflect;
        Self {
            normal: has(RenderMode::Normal)
                .then(|| NativeGradient::new(stops, false, RenderMode::Normal))
                .flatten(),
            normal_reverse: (has(RenderMode::Normal) && reversed)
                .then(|| NativeGradient::new(stops, true, RenderMode::Normal))
                .flatten(),
            alpha: has(RenderMode::AlphaMask)
                .then(|| NativeGradient::new(stops, false, RenderMode::AlphaMask))
                .flatten(),
            alpha_reverse: (has(RenderMode::AlphaMask) && reversed)
                .then(|| NativeGradient::new(stops, true, RenderMode::AlphaMask))
                .flatten(),
            luminance: has(RenderMode::LuminanceMask)
                .then(|| NativeGradient::new(stops, false, RenderMode::LuminanceMask))
                .flatten(),
            luminance_reverse: (has(RenderMode::LuminanceMask) && reversed)
                .then(|| NativeGradient::new(stops, true, RenderMode::LuminanceMask))
                .flatten(),
        }
    }

    const fn get(&self, mode: RenderMode, reverse: bool) -> Option<&NativeGradient> {
        match (mode, reverse) {
            (RenderMode::Normal, false) => self.normal.as_ref(),
            (RenderMode::Normal, true) => self.normal_reverse.as_ref(),
            (RenderMode::AlphaMask, false) => self.alpha.as_ref(),
            (RenderMode::AlphaMask, true) => self.alpha_reverse.as_ref(),
            (RenderMode::LuminanceMask, false) => self.luminance.as_ref(),
            (RenderMode::LuminanceMask, true) => self.luminance_reverse.as_ref(),
        }
    }
}
struct NativeGradient(*const c_void);
impl NativeGradient {
    fn new(stops: &[image::GradientStop], reverse: bool, mode: RenderMode) -> Option<Self> {
        let iter = stops
            .iter()
            .map(|stop| (stop.offset, mask_color(stop.color, mode)));
        if reverse {
            Self::from_iter(
                iter.rev().map(|(offset, color)| (1.0 - offset, color)),
                mode,
            )
        } else {
            Self::from_iter(iter, mode)
        }
    }

    fn from_iter(
        iter: impl ExactSizeIterator<Item = (f64, Color)>,
        mode: RenderMode,
    ) -> Option<Self> {
        let count = iter.len();
        if count == 0 {
            return None;
        }
        let mut components = Vec::with_capacity(count * 4);
        let mut locations = Vec::with_capacity(count);
        for (offset, mut color) in iter {
            if matches!(mode, RenderMode::Normal) {
                color = display_color(color);
            }
            components.extend([color.red, color.green, color.blue, color.alpha]);
            locations.push(offset);
        }
        // SAFETY: Core Graphics copies both temporary arrays.
        unsafe {
            let color_space = CGColorSpaceCreateWithName(match mode {
                RenderMode::Normal => kCGColorSpaceSRGB,
                RenderMode::AlphaMask | RenderMode::LuminanceMask => kCGColorSpaceLinearSRGB,
            });
            if color_space.is_null() {
                return None;
            }
            let gradient = CGGradientCreateWithColorComponents(
                color_space,
                components.as_ptr(),
                locations.as_ptr(),
                count,
            );
            CGColorSpaceRelease(color_space);
            (!gradient.is_null()).then_some(Self(gradient))
        }
    }
}
impl Drop for NativeGradient {
    fn drop(&mut self) {
        // SAFETY: This gradient was returned with ownership by Core Graphics.
        unsafe { CGGradientRelease(self.0) }
    }
}

define_class!(
    #[unsafe(super(NSImageRep))]
    #[name = "MacViewVectorImageRep"]
    #[ivars = VectorImageRepIvars]
    struct VectorImageRep;
    impl VectorImageRep {
        #[unsafe(method(draw))]
        fn _draw(&self) -> Bool { let size = self.ivars().document.size(); self.draw_in(Rect { origin: CgPoint { x: 0.0, y: 0.0 }, size: Size { width: size.width, height: size.height } }) }
    }
);
impl VectorImageRep {
    fn draw_in(&self, rect: Rect) -> Bool {
        // SAFETY: AppKit supplies a live graphics context during drawing.
        unsafe {
            let graphics: *mut Object = msg_send![class!(NSGraphicsContext), currentContext];
            if graphics.is_null() {
                return Bool::NO;
            }
            let context: *mut c_void = msg_send![graphics, CGContext];
            if context.is_null() {
                return Bool::NO;
            }
            let flipped: Bool = msg_send![graphics, isFlipped];
            CGContextSaveGState(context);
            CGContextTranslateCTM(context, rect.origin.x, rect.origin.y);
            if !flipped.as_bool() {
                CGContextTranslateCTM(context, 0.0, rect.size.height);
                CGContextScaleCTM(context, 1.0, -1.0);
            }
            render_fitted(
                context,
                &self.ivars().document,
                &self.ivars().paths,
                &self.ivars().paints,
                rect.size,
            );
            CGContextRestoreGState(context);
            Bool::YES
        }
    }
}

pub(crate) fn create_vector_image(document: VectorImage) -> Retained<Object> {
    let vector_size = document.size();
    let size = Size {
        width: vector_size.width,
        height: vector_size.height,
    };
    let paths = document
        .paths()
        .map(|(_, path)| NativePath::new(path).expect("Core Graphics path allocation failed"))
        .collect();
    let mut paint_modes = vec![0; document.paints().len()];
    collect_paint_modes(document.commands(), RenderMode::Normal, &mut paint_modes);
    let paints = document
        .paints()
        .map(|(id, paint)| PreparedPaint::new(paint, paint_modes[id.index()]))
        .collect();
    // SAFETY: The registered representation owns the display list and prepared resources.
    unsafe {
        let representation: Allocated<VectorImageRep> = msg_send![VectorImageRep::class(), alloc];
        let representation: Retained<VectorImageRep> = msg_send![
            super(representation.set_ivars(VectorImageRepIvars {
                document,
                paths,
                paints
            })),
            init
        ];
        let _: () = msg_send![&*representation, setSize: size];
        let _: () = msg_send![&*representation, setAlpha: Bool::YES];
        let _: () = msg_send![&*representation, setOpaque: Bool::NO];
        let image: Allocated<Object> = msg_send![class!(NSImage), alloc];
        let image: Retained<Object> = msg_send![image, initWithSize: size];
        let _: () = msg_send![&*image, setCacheMode: NS_IMAGE_CACHE_NEVER];
        let _: () = msg_send![&*image, addRepresentation: representation.as_ptr()];
        image
    }
}

unsafe fn render_fitted(
    context: *mut c_void,
    document: &VectorImage,
    paths: &[NativePath],
    paints: &[PreparedPaint],
    bounds: Size,
) {
    let size = document.size();
    if size.width <= 0.0 || size.height <= 0.0 {
        return;
    }
    let scale = (bounds.width.max(1.0) / size.width)
        .min(bounds.height.max(1.0) / size.height)
        .max(f64::EPSILON);
    let x = (bounds.width - size.width * scale) / 2.0;
    let y = (bounds.height - size.height * scale) / 2.0;
    // SAFETY: The caller supplies a live context and all decoder transforms are finite.
    unsafe {
        CGContextSaveGState(context);
        CGContextTranslateCTM(context, x, y);
        CGContextScaleCTM(context, scale, scale);
        let clip = CGContextGetClipBoundingBox(context);
        render(context, document, paths, paints, clip);
        CGContextRestoreGState(context);
    }
}

unsafe fn render(
    context: *mut c_void,
    document: &VectorImage,
    paths: &[NativePath],
    paints: &[PreparedPaint],
    clip: Rect,
) {
    let mut index = 0;
    // SAFETY: The decoder emits balanced scopes and matching resource indexes.
    unsafe {
        render_commands(
            context,
            document.commands(),
            &mut index,
            paths,
            paints,
            clip,
            RenderMode::Normal,
        );
    }
}

#[derive(Clone, Copy)]
enum RenderMode {
    Normal,
    AlphaMask,
    LuminanceMask,
}
impl RenderMode {
    const fn bit(self) -> u8 {
        match self {
            Self::Normal => 1,
            Self::AlphaMask => 2,
            Self::LuminanceMask => 4,
        }
    }
}

fn collect_paint_modes(commands: &[DrawCommand], mode: RenderMode, modes: &mut [u8]) {
    for command in commands {
        match command {
            DrawCommand::PushScope {
                mask: Some(mask), ..
            } => collect_paint_modes(
                &mask.commands,
                match mask.mask_type {
                    MaskType::Alpha => RenderMode::AlphaMask,
                    MaskType::Luminance => RenderMode::LuminanceMask,
                },
                modes,
            ),
            DrawCommand::Fill { paint, .. } | DrawCommand::Stroke { paint, .. } => {
                modes[paint.index()] |= mode.bit();
            }
            DrawCommand::PushScope { mask: None, .. } | DrawCommand::PopScope => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn render_commands(
    context: *mut c_void,
    commands: &[DrawCommand],
    index: &mut usize,
    paths: &[NativePath],
    paints: &[PreparedPaint],
    clip: Rect,
    mode: RenderMode,
) {
    while let Some(command) = commands.get(*index) {
        *index += 1;
        match command {
            DrawCommand::PushScope {
                opacity,
                clips,
                mask,
                ..
            } => {
                // SAFETY: The caller supplies a live context and prepared paths are immutable.
                unsafe {
                    CGContextSaveGState(context);
                    for item in clips {
                        CGContextBeginPath(context);
                        CGContextSaveGState(context);
                        CGContextConcatCTM(context, item.transform);
                        CGContextAddPath(context, paths[item.path.index()].0);
                        CGContextRestoreGState(context);
                        match item.rule {
                            FillRule::NonZero => CGContextClip(context),
                            FillRule::EvenOdd => CGContextEOClip(context),
                        }
                    }
                    if let Some(mask) = mask {
                        CGContextBeginPath(context);
                        CGContextSaveGState(context);
                        CGContextConcatCTM(context, mask.region.transform);
                        CGContextAddPath(context, paths[mask.region.path.index()].0);
                        CGContextRestoreGState(context);
                        CGContextClip(context);
                    }
                    let transparent = *opacity < 1.0 || mask.is_some();
                    if transparent {
                        CGContextSetAlpha(context, *opacity);
                        CGContextBeginTransparencyLayer(context, null());
                    }
                    if let Some(mask) = mask {
                        let mut mask_index = 0;
                        CGContextSaveGState(context);
                        CGContextSetBlendMode(context, 17); // kCGBlendModeCopy
                        render_commands(
                            context,
                            &mask.commands,
                            &mut mask_index,
                            paths,
                            paints,
                            clip,
                            match mask.mask_type {
                                MaskType::Alpha => RenderMode::AlphaMask,
                                MaskType::Luminance => RenderMode::LuminanceMask,
                            },
                        );
                        CGContextRestoreGState(context);
                        CGContextSaveGState(context);
                        CGContextSetBlendMode(context, 18); // kCGBlendModeSourceIn
                        CGContextBeginTransparencyLayer(context, null());
                        CGContextSetBlendMode(context, 0); // kCGBlendModeNormal
                        render_commands(context, commands, index, paths, paints, clip, mode);
                        CGContextEndTransparencyLayer(context);
                        CGContextRestoreGState(context);
                    } else {
                        render_commands(context, commands, index, paths, paints, clip, mode);
                    }
                    if transparent {
                        CGContextEndTransparencyLayer(context);
                    }
                    CGContextRestoreGState(context);
                }
            }
            DrawCommand::PopScope => return,
            DrawCommand::Fill {
                path,
                paint,
                transform,
                rule,
                bounds,
            } if intersects(*bounds, clip) => {
                // SAFETY: Resource IDs were produced together with these prepared resources.
                unsafe {
                    draw_path(
                        context,
                        paths,
                        paints,
                        path.index(),
                        paint.index(),
                        *transform,
                        None,
                        *rule,
                        mode,
                    );
                }
            }
            DrawCommand::Stroke {
                path,
                paint,
                transform,
                style,
                bounds,
            } if intersects(*bounds, clip) => {
                // SAFETY: Resource IDs were produced together with these prepared resources.
                unsafe {
                    draw_path(
                        context,
                        paths,
                        paints,
                        path.index(),
                        paint.index(),
                        *transform,
                        Some(style),
                        FillRule::NonZero,
                        mode,
                    );
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn draw_path(
    context: *mut c_void,
    paths: &[NativePath],
    paints: &[PreparedPaint],
    path: usize,
    paint: usize,
    transform: Transform,
    stroke: Option<&image::StrokeStyle>,
    rule: FillRule,
    mode: RenderMode,
) {
    // SAFETY: Prepared indexes correspond to validated display-list resource IDs.
    unsafe {
        CGContextSaveGState(context);
        CGContextConcatCTM(context, transform);
        CGContextBeginPath(context);
        CGContextAddPath(context, paths[path].0);
        if let Some(style) = stroke {
            CGContextSetLineWidth(context, if style.width > 0.0 { style.width } else { 1.0 });
            CGContextSetLineCap(
                context,
                match style.line_cap {
                    LineCap::Butt => LINE_CAP_BUTT,
                    LineCap::Round => LINE_CAP_ROUND,
                    LineCap::Square => LINE_CAP_SQUARE,
                },
            );
            CGContextSetLineJoin(
                context,
                match style.line_join {
                    LineJoin::Miter => LINE_JOIN_MITER,
                    LineJoin::Round => LINE_JOIN_ROUND,
                    LineJoin::Bevel => LINE_JOIN_BEVEL,
                },
            );
            CGContextSetMiterLimit(context, style.miter_limit);
            CGContextSetLineDash(
                context,
                style.dash_offset,
                style.dash_array.as_ptr(),
                style.dash_array.len(),
            );
        }
        let prepared = &paints[paint];
        if let PreparedPaint::Solid(color) = prepared {
            let color = mask_color(*color, mode);
            if stroke.is_some() {
                CGContextSetRGBStrokeColor(
                    context,
                    color.red,
                    color.green,
                    color.blue,
                    color.alpha,
                );
            } else {
                CGContextSetRGBFillColor(context, color.red, color.green, color.blue, color.alpha);
            }
            CGContextDrawPath(
                context,
                if stroke.is_some() {
                    DRAW_PATH_STROKE
                } else if rule == FillRule::EvenOdd {
                    DRAW_PATH_EVEN_ODD_FILL
                } else {
                    DRAW_PATH_FILL
                },
            );
        } else {
            if stroke.is_some() {
                CGContextReplacePathWithStrokedPath(context);
                CGContextClip(context);
            } else {
                match rule {
                    FillRule::NonZero => CGContextClip(context),
                    FillRule::EvenOdd => CGContextEOClip(context),
                }
            }
            draw_gradient(context, prepared, mode);
        }
        CGContextRestoreGState(context);
    }
}

unsafe fn draw_gradient(context: *mut c_void, paint: &PreparedPaint, mode: RenderMode) {
    // SAFETY: Prepared gradients and the drawing context remain alive for this call.
    unsafe {
        match paint {
            PreparedPaint::Linear {
                gradients,
                start,
                end,
                transform,
                spread,
            } => {
                let Some(g) = gradients.get(mode, false) else {
                    return;
                };
                CGContextSaveGState(context);
                CGContextConcatCTM(context, *transform);
                if *spread == SpreadMethod::Pad {
                    CGContextDrawLinearGradient(
                        context,
                        g.0,
                        *start,
                        *end,
                        GRADIENT_DRAWS_BEFORE_START | GRADIENT_DRAWS_AFTER_END,
                    );
                } else {
                    let clip = CGContextGetClipBoundingBox(context);
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    let length_squared = dx * dx + dy * dy;
                    if length_squared > 0.0 {
                        let project = |x: f64, y: f64| {
                            ((x - start.x) * dx + (y - start.y) * dy) / length_squared
                        };
                        let values = [
                            project(clip.origin.x, clip.origin.y),
                            project(clip.origin.x + clip.size.width, clip.origin.y),
                            project(clip.origin.x, clip.origin.y + clip.size.height),
                            project(
                                clip.origin.x + clip.size.width,
                                clip.origin.y + clip.size.height,
                            ),
                        ];
                        let first = values.into_iter().fold(f64::INFINITY, f64::min).floor() as i64;
                        let last =
                            values.into_iter().fold(f64::NEG_INFINITY, f64::max).ceil() as i64;
                        for index in first.max(-4096)..last.min(4096) {
                            let gradient =
                                if *spread == SpreadMethod::Reflect && index.rem_euclid(2) == 1 {
                                    gradients.get(mode, true).unwrap_or(g)
                                } else {
                                    g
                                };
                            CGContextDrawLinearGradient(
                                context,
                                gradient.0,
                                image::Point {
                                    x: start.x + dx * index as f64,
                                    y: start.y + dy * index as f64,
                                },
                                image::Point {
                                    x: start.x + dx * (index + 1) as f64,
                                    y: start.y + dy * (index + 1) as f64,
                                },
                                0,
                            );
                        }
                    }
                }
                CGContextRestoreGState(context);
            }
            PreparedPaint::Radial {
                gradients,
                center,
                focal,
                radius,
                transform,
                spread,
            } => {
                let Some(g) = gradients.get(mode, false) else {
                    return;
                };
                CGContextSaveGState(context);
                CGContextConcatCTM(context, *transform);
                if *spread == SpreadMethod::Pad {
                    CGContextDrawRadialGradient(
                        context,
                        g.0,
                        *focal,
                        0.0,
                        *center,
                        *radius,
                        GRADIENT_DRAWS_BEFORE_START | GRADIENT_DRAWS_AFTER_END,
                    );
                } else if *radius > 0.0 {
                    let clip = CGContextGetClipBoundingBox(context);
                    let distances = [
                        (clip.origin.x - center.x).hypot(clip.origin.y - center.y),
                        (clip.origin.x + clip.size.width - center.x)
                            .hypot(clip.origin.y - center.y),
                        (clip.origin.x - center.x)
                            .hypot(clip.origin.y + clip.size.height - center.y),
                        (clip.origin.x + clip.size.width - center.x)
                            .hypot(clip.origin.y + clip.size.height - center.y),
                    ];
                    let count = (distances.into_iter().fold(0.0, f64::max) / radius)
                        .ceil()
                        .min(4096.0) as usize;
                    for index in 0..count {
                        let gradient = if *spread == SpreadMethod::Reflect && index % 2 == 1 {
                            gradients.get(mode, true).unwrap_or(g)
                        } else {
                            g
                        };
                        CGContextDrawRadialGradient(
                            context,
                            gradient.0,
                            *center,
                            *radius * index as f64,
                            *center,
                            *radius * (index + 1) as f64,
                            0,
                        );
                    }
                }
                CGContextRestoreGState(context);
            }
            _ => {}
        }
    }
}

fn intersects(bounds: image::Rect, clip: Rect) -> bool {
    bounds.x + bounds.width >= clip.origin.x
        && bounds.x <= clip.origin.x + clip.size.width
        && bounds.y + bounds.height >= clip.origin.y
        && bounds.y <= clip.origin.y + clip.size.height
}

fn mask_color(color: Color, mode: RenderMode) -> Color {
    match mode {
        RenderMode::Normal => color,
        RenderMode::AlphaMask => Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: color.alpha,
            color_space: VectorColorSpace::LinearSrgb,
        },
        RenderMode::LuminanceMask => {
            let [red, green, blue, alpha] = linear_components(color);
            Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: alpha * (red * 0.2126 + green * 0.7152 + blue * 0.0722),
                color_space: VectorColorSpace::LinearSrgb,
            }
        }
    }
}

fn display_color(mut color: Color) -> Color {
    if color.color_space == VectorColorSpace::LinearSrgb {
        color.red = color.red.max(0.0).powf(1.0 / 2.2);
        color.green = color.green.max(0.0).powf(1.0 / 2.2);
        color.blue = color.blue.max(0.0).powf(1.0 / 2.2);
        color.color_space = VectorColorSpace::Srgb;
    }
    color
}

fn linear_components(color: Color) -> [f64; 4] {
    let convert = |value: f64| match color.color_space {
        VectorColorSpace::Srgb => value.max(0.0).powf(2.2),
        VectorColorSpace::LinearSrgb => value,
    };
    [
        convert(color.red),
        convert(color.green),
        convert(color.blue),
        color.alpha,
    ]
}

/// Fills a valid Core Graphics context with an opaque white background.
///
/// # Safety
///
/// `context` must remain valid for the duration of the call.
pub unsafe fn fill_white_background(context: *mut c_void, bounds: Size) {
    // SAFETY: The caller guarantees that the context is valid.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render_pixels(document: &VectorImage, width: usize, height: usize) -> Vec<u8> {
        let paths = document
            .paths()
            .map(|(_, path)| NativePath::new(path).expect("native path"))
            .collect::<Vec<_>>();
        let paints = document
            .paints()
            .map(|(_, paint)| PreparedPaint::new(paint, 1 | 2 | 4))
            .collect::<Vec<_>>();
        let mut pixels = vec![0_u8; width * height * 4];
        // SAFETY: The backing allocation remains live until after the context is released.
        unsafe {
            let color_space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
            let context = CGBitmapContextCreate(
                pixels.as_mut_ptr().cast(),
                width,
                height,
                8,
                width * 4,
                color_space,
                1 | (4 << 12),
            );
            assert!(!context.is_null());
            render_fitted(
                context,
                document,
                &paths,
                &paints,
                Size {
                    width: width as f64,
                    height: height as f64,
                },
            );
            CGContextRelease(context);
            CGColorSpaceRelease(color_space);
        }
        pixels
    }

    #[test]
    fn renders_alpha_and_luminance_masks_into_bitmap() {
        for svg in [
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
              <defs><mask id="half" mask-type="alpha" maskContentUnits="objectBoundingBox">
                <rect width=".5" height="1" fill="white"/>
              </mask></defs>
              <rect width="20" height="10" fill="red" mask="url(#half)"/>
            </svg>"##
                .as_slice(),
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
              <defs><mask id="half" mask-type="luminance">
                <rect width="20" height="10" fill="white"/>
                <rect x="10" width="10" height="10" fill="black"/>
              </mask></defs>
              <rect width="20" height="10" fill="red" mask="url(#half)"/>
            </svg>"##
                .as_slice(),
        ] {
            let document = image::decode_vector(svg).expect("masked SVG");
            let pixels = render_pixels(&document, 20, 10);
            assert!(pixels[(5 * 20 + 5) * 4 + 3] > 240);
            assert!(pixels[(5 * 20 + 15) * 4 + 3] < 16);
        }
    }

    #[test]
    fn renders_transformed_mask_with_view_box() {
        let document = image::decode_vector(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 100 100">
              <mask id="cutout">
                <rect width="100" height="100" fill="white"/>
                <rect x="40" width="20" height="100" fill="black"/>
              </mask>
              <g transform="translate(10) scale(.8)">
                <rect width="100" height="100" fill="white" mask="url(#cutout)"/>
              </g>
            </svg>"##,
        )
        .expect("transformed masked SVG");
        let pixels = render_pixels(&document, 200, 200);
        assert!(pixels[(100 * 200 + 40) * 4 + 3] > 240);
        assert!(pixels[(100 * 200 + 100) * 4 + 3] < 16);
    }

    #[test]
    fn renders_svg_gradients_in_srgb() {
        let document = image::decode_vector(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="1">
              <linearGradient id="gradient"><stop/><stop offset="1" stop-color="white"/></linearGradient>
              <rect width="256" height="1" fill="url(#gradient)"/>
            </svg>"##,
        )
        .expect("gradient SVG");
        let pixels = render_pixels(&document, 256, 1);
        let midpoint = pixels[128 * 4];
        assert!((120..=136).contains(&midpoint), "midpoint was {midpoint}");
    }
}
