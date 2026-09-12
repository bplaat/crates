/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::MAX_ITEMS;
use super::geometry::{
    commands_bounds, commands_object_bounds, inverse_transform, object_bbox_length,
    parse_mask_type, stroke_extent, transform_bounds, transform_segment,
};
use super::path::{rect_path, shape_path};
use super::style::{
    PaintValue, Style, local_reference, parse_color, parse_style, skips_unsupported_subtree,
};
use super::values::{
    compatible_length, length, parse_transform, parse_view_box, unit_number, view_box_transform,
};
use super::xml::XmlDocument;
use crate::vector::path_bounds;
use crate::{
    DrawCommand, FillRule, Mask, Paint, Point, Rect, Size, Transform, VectorDecodeError,
    VectorFormat, VectorImage,
};

pub(super) struct Builder<'a> {
    document: &'a XmlDocument<'a>,
    image: VectorImage,
    references: Vec<usize>,
    items: usize,
}

impl<'a> Builder<'a> {
    pub(super) fn new(document: &'a XmlDocument<'a>) -> Result<Self, VectorDecodeError> {
        let root = &document.elements[document.root];
        let view_box = root.attr("viewBox").map(parse_view_box).transpose()?;
        let default = view_box.map_or((512.0, 512.0), |value| (value.width, value.height));
        let width = compatible_length(root.attr("width"), default.0)?.unwrap_or(default.0);
        let height = compatible_length(root.attr("height"), default.1)?.unwrap_or(default.1);
        if width <= 0.0 || height <= 0.0 {
            return Err(VectorDecodeError::InvalidData);
        }
        Ok(Self {
            document,
            image: VectorImage::new(VectorFormat::Svg, Size { width, height }),
            references: Vec::new(),
            items: 0,
        })
    }

    pub(super) fn build(mut self) -> Result<VectorImage, VectorDecodeError> {
        let root = &self.document.elements[self.document.root];
        let mut transform = Transform::IDENTITY;
        let view_box = root.attr("viewBox").map(parse_view_box).transpose()?;
        if let Some(view_box) = view_box {
            transform = view_box_transform(
                view_box,
                self.image.size().width,
                self.image.size().height,
                root.attr("preserveAspectRatio"),
            )?;
        }
        let viewport = view_box.map_or(self.image.size(), |view_box| Size {
            width: view_box.width,
            height: view_box.height,
        });
        self.render(
            self.document.root,
            &Style::default(),
            transform,
            viewport,
            true,
        )?;
        if self.image.commands().len() > MAX_ITEMS {
            return Err(VectorDecodeError::ResourceLimit);
        }
        Ok(self.image)
    }

    fn render(
        &mut self,
        index: usize,
        inherited: &Style,
        parent_transform: Transform,
        viewport: Size,
        is_root: bool,
    ) -> Result<(), VectorDecodeError> {
        let command_start = self.image.command_len();
        let (path_start, paint_start) = self.image.resource_lengths();
        let item_start = self.items;
        match self.render_inner(index, inherited, parent_transform, viewport, is_root) {
            Err(VectorDecodeError::UnsupportedFeature(_)) => {
                self.image.drain_commands(command_start);
                self.image.truncate_resources(path_start, paint_start);
                self.items = item_start;
                Ok(())
            }
            result => result,
        }
    }

    fn render_inner(
        &mut self,
        index: usize,
        inherited: &Style,
        parent_transform: Transform,
        viewport: Size,
        is_root: bool,
    ) -> Result<(), VectorDecodeError> {
        if self.references.len() >= 64 {
            return Err(VectorDecodeError::ResourceLimit);
        }
        let element = &self.document.elements[index];
        if !element.is_svg || skips_unsupported_subtree(element.name) {
            return Ok(());
        }
        if matches!(
            element.name,
            "defs" | "linearGradient" | "radialGradient" | "stop" | "clipPath" | "mask"
        ) {
            return Ok(());
        }
        let style = parse_style(element, inherited)?;
        if !style.displayed {
            return Ok(());
        }
        let mut local = element
            .attr("transform")
            .map(parse_transform)
            .transpose()?
            .unwrap_or(Transform::IDENTITY);
        let mut child_viewport = viewport;
        let mut viewport_clip = if element.name == "svg" && is_root {
            let bounds = Rect {
                x: 0.0,
                y: 0.0,
                width: self.image.size().width,
                height: self.image.size().height,
            };
            Some(self.viewport_clip(bounds, Transform::IDENTITY)?)
        } else {
            None
        };
        if element.name == "svg" && !is_root {
            let x = element
                .attr("x")
                .map(|value| length(value, viewport.width))
                .transpose()?
                .unwrap_or(0.0);
            let y = element
                .attr("y")
                .map(|value| length(value, viewport.height))
                .transpose()?
                .unwrap_or(0.0);
            let width = element
                .attr("width")
                .map(|value| length(value, viewport.width))
                .transpose()?
                .unwrap_or(viewport.width);
            let height = element
                .attr("height")
                .map(|value| length(value, viewport.height))
                .transpose()?
                .unwrap_or(viewport.height);
            if width <= 0.0 || height <= 0.0 {
                return Ok(());
            }
            let viewport_transform = Transform {
                e: x,
                f: y,
                ..Transform::IDENTITY
            }
            .then(parent_transform);
            viewport_clip = Some(self.viewport_clip(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                },
                viewport_transform,
            )?);
            let view = element.attr("viewBox").map(parse_view_box).transpose()?;
            child_viewport = view.map_or(Size { width, height }, |view| Size {
                width: view.width,
                height: view.height,
            });
            let view_transform = if let Some(view) = view {
                view_box_transform(view, width, height, element.attr("preserveAspectRatio"))?
            } else {
                Transform::IDENTITY
            };
            local = local.then(view_transform).then(Transform {
                e: x,
                f: y,
                ..Transform::IDENTITY
            });
        }
        let transform = local.then(parent_transform);
        if element.name == "use" {
            let href = element
                .attr("href")
                .or_else(|| element.attr_prefixed("xlink", "href"))
                .ok_or(VectorDecodeError::InvalidData)?;
            let id = local_reference(href)?;
            let target = *self
                .document
                .ids
                .get(id)
                .ok_or(VectorDecodeError::InvalidData)?;
            if self.references.contains(&target) {
                return Err(VectorDecodeError::InvalidData);
            }
            let x = element
                .attr("x")
                .map(|v| length(v, viewport.width))
                .transpose()?
                .unwrap_or(0.0);
            let y = element
                .attr("y")
                .map(|v| length(v, viewport.height))
                .transpose()?
                .unwrap_or(0.0);
            self.references.push(target);
            let result = self.render(
                target,
                &style,
                Transform {
                    e: x,
                    f: y,
                    ..Transform::IDENTITY
                }
                .then(transform),
                viewport,
                false,
            );
            self.references.pop();
            return result;
        }
        let mut clips = viewport_clip.into_iter().collect::<Vec<_>>();
        if let Some(reference) = &style.clip_path {
            match self.resolve_clip(reference, transform, viewport) {
                Ok(clip) => clips.push(clip),
                Err(VectorDecodeError::UnsupportedFeature(_)) => {}
                Err(error) => return Err(error),
            }
        }
        let command_start = self.image.command_len();
        if let Some(path) = shape_path(element, viewport)? {
            if style.visible && !path.is_empty() {
                self.items = self
                    .items
                    .checked_add(path.len())
                    .ok_or(VectorDecodeError::ResourceLimit)?;
                if self.items > MAX_ITEMS {
                    return Err(VectorDecodeError::ResourceLimit);
                }
                let local_bounds = path_bounds(&path).ok_or(VectorDecodeError::InvalidData)?;
                let bounds = transform_bounds(local_bounds, transform);
                let fill =
                    self.resolve_paint(&style.fill, style.fill_opacity, local_bounds, viewport)?;
                let stroke = self.resolve_paint(
                    &style.stroke,
                    style.stroke_opacity,
                    local_bounds,
                    viewport,
                )?;
                if fill.is_some() || stroke.is_some() {
                    let path_id = self.image.add_path(path);
                    if let Some(paint) = fill {
                        let paint = self.image.add_paint(paint);
                        self.image.push(DrawCommand::Fill {
                            path: path_id,
                            paint,
                            transform,
                            rule: style.fill_rule,
                            bounds,
                        });
                    }
                    if let Some(paint) = stroke {
                        let paint = self.image.add_paint(paint);
                        let stroke_bounds =
                            bounds.expand(stroke_extent(&style.stroke_style, transform));
                        self.image.push(DrawCommand::Stroke {
                            path: path_id,
                            paint,
                            transform,
                            style: style.stroke_style.clone(),
                            bounds: stroke_bounds,
                        });
                    }
                }
            }
        } else if !matches!(element.name, "svg" | "g" | "a" | "symbol") {
            return Err(VectorDecodeError::UnsupportedFeature("element"));
        }
        for &child in &element.children {
            self.render(child, &style, transform, child_viewport, false)?;
        }
        let content_bounds = commands_bounds(&self.image.commands()[command_start..]);
        let mask = if let Some(reference) = &style.mask {
            let inverse = inverse_transform(transform).ok_or(VectorDecodeError::InvalidData)?;
            let local_bounds = commands_object_bounds(
                &self.image,
                &self.image.commands()[command_start..],
                inverse,
            )
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            });
            match self.resolve_mask(reference, transform, local_bounds, viewport) {
                Ok(mask) => Some(mask),
                Err(VectorDecodeError::UnsupportedFeature(_)) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        let scope = style.opacity < 1.0 || !clips.is_empty() || mask.is_some();
        if scope {
            let bounds = content_bounds.unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            });
            self.image.insert_command(
                command_start,
                DrawCommand::PushScope {
                    opacity: style.opacity,
                    clips,
                    mask,
                    bounds,
                },
            );
            self.image.push(DrawCommand::PopScope);
        }
        Ok(())
    }

    fn viewport_clip(
        &mut self,
        local_bounds: Rect,
        transform: Transform,
    ) -> Result<crate::Clip, VectorDecodeError> {
        let path = rect_path(
            local_bounds.x,
            local_bounds.y,
            local_bounds.width,
            local_bounds.height,
            0.0,
            0.0,
        );
        self.items = self
            .items
            .checked_add(path.len())
            .ok_or(VectorDecodeError::ResourceLimit)?;
        if self.items > MAX_ITEMS {
            return Err(VectorDecodeError::ResourceLimit);
        }
        Ok(crate::Clip {
            path: self.image.add_path(path),
            transform,
            rule: FillRule::NonZero,
            bounds: transform_bounds(local_bounds, transform),
        })
    }

    fn resolve_paint(
        &self,
        value: &PaintValue,
        opacity: f64,
        bounds: Rect,
        viewport: Size,
    ) -> Result<Option<Paint>, VectorDecodeError> {
        match value {
            PaintValue::None => Ok(None),
            PaintValue::Color(color) => {
                let mut color = *color;
                color.alpha *= opacity;
                Ok(Some(Paint::Solid(color)))
            }
            PaintValue::Reference(reference) => {
                let Ok(reference) = local_reference(reference) else {
                    return Ok(None);
                };
                let index = *self
                    .document
                    .ids
                    .get(reference)
                    .ok_or(VectorDecodeError::InvalidData)?;
                if !matches!(
                    self.document.elements[index].name,
                    "linearGradient" | "radialGradient"
                ) {
                    return Ok(None);
                }
                self.gradient(index, bounds, opacity, viewport, &mut Vec::new())
                    .map(Some)
            }
        }
    }

    fn gradient(
        &self,
        index: usize,
        bounds: Rect,
        opacity: f64,
        viewport: Size,
        chain: &mut Vec<usize>,
    ) -> Result<Paint, VectorDecodeError> {
        if chain.len() >= 64 {
            return Err(VectorDecodeError::ResourceLimit);
        }
        if chain.contains(&index) {
            return Err(VectorDecodeError::InvalidData);
        }
        chain.push(index);
        let element = &self.document.elements[index];
        if !matches!(element.name, "linearGradient" | "radialGradient") {
            return Err(VectorDecodeError::InvalidData);
        }
        let inherited = if let Some(reference) = element
            .attr("href")
            .or_else(|| element.attr_prefixed("xlink", "href"))
        {
            let target = *self
                .document
                .ids
                .get(local_reference(reference)?)
                .ok_or(VectorDecodeError::InvalidData)?;
            Some(self.gradient(target, bounds, opacity, viewport, chain)?)
        } else {
            None
        };
        chain.pop();
        let mut stops = Vec::new();
        let mut last = 0.0;
        for &child in &element.children {
            let stop = &self.document.elements[child];
            if stop.name != "stop" {
                continue;
            }
            if stops.len() >= 65_536 {
                return Err(VectorDecodeError::ResourceLimit);
            }
            let offset = stop
                .attr("offset")
                .map(unit_number)
                .transpose()?
                .unwrap_or(0.0)
                .max(last);
            last = offset;
            let mut stop_color = stop.attr("stop-color").unwrap_or("black");
            let mut stop_opacity = stop
                .attr("stop-opacity")
                .map(unit_number)
                .transpose()?
                .unwrap_or(1.0);
            if let Some(inline) = stop.attr("style") {
                for declaration in inline.split(';') {
                    if let Some((name, value)) = declaration.split_once(':') {
                        match name.trim() {
                            "stop-color" => stop_color = value.trim(),
                            "stop-opacity" => stop_opacity = unit_number(value.trim())?,
                            _ => {}
                        }
                    }
                }
            }
            let mut color = parse_color(stop_color, Style::default().color)?;
            color.alpha *= stop_opacity * opacity;
            stops.push(crate::GradientStop { offset, color });
        }
        if stops.is_empty()
            && let Some(
                Paint::LinearGradient { stops: value, .. }
                | Paint::RadialGradient { stops: value, .. },
            ) = &inherited
        {
            stops = value.clone();
        }
        if stops.is_empty() {
            return Err(VectorDecodeError::InvalidData);
        }
        let object_units = match element.attr("gradientUnits").unwrap_or("objectBoundingBox") {
            "objectBoundingBox" => true,
            "userSpaceOnUse" => false,
            _ => return Err(VectorDecodeError::InvalidData),
        };
        let base = if object_units {
            Transform {
                a: bounds.width,
                b: 0.0,
                c: 0.0,
                d: bounds.height,
                e: bounds.x,
                f: bounds.y,
            }
        } else {
            Transform::IDENTITY
        };
        let transform = base.then(
            element
                .attr("gradientTransform")
                .map(parse_transform)
                .transpose()?
                .unwrap_or(Transform::IDENTITY),
        );
        let spread = match element.attr("spreadMethod").unwrap_or("pad") {
            "pad" => crate::SpreadMethod::Pad,
            "reflect" => crate::SpreadMethod::Reflect,
            "repeat" => crate::SpreadMethod::Repeat,
            _ => return Err(VectorDecodeError::InvalidData),
        };
        let x_axis = if object_units { 1.0 } else { viewport.width };
        let y_axis = if object_units { 1.0 } else { viewport.height };
        let radius_axis = if object_units {
            1.0
        } else {
            viewport.width.hypot(viewport.height) / std::f64::consts::SQRT_2
        };
        let coordinate = |name: &str, default: &str, axis: f64| {
            length(element.attr(name).unwrap_or(default), axis)
        };
        if element.name == "linearGradient" {
            let fallback = match inherited {
                Some(Paint::LinearGradient { start, end, .. }) => Some((start, end)),
                _ => None,
            };
            let start = Point {
                x: if element.attr("x1").is_some() {
                    coordinate("x1", "0%", x_axis)?
                } else {
                    fallback.map_or(0.0, |v| v.0.x)
                },
                y: if element.attr("y1").is_some() {
                    coordinate("y1", "0%", y_axis)?
                } else {
                    fallback.map_or(0.0, |v| v.0.y)
                },
            };
            let end = Point {
                x: if element.attr("x2").is_some() {
                    coordinate("x2", "100%", x_axis)?
                } else {
                    fallback.map_or(x_axis, |v| v.1.x)
                },
                y: if element.attr("y2").is_some() {
                    coordinate("y2", "0%", y_axis)?
                } else {
                    fallback.map_or(0.0, |v| v.1.y)
                },
            };
            Ok(Paint::LinearGradient {
                start,
                end,
                stops,
                spread,
                transform,
            })
        } else {
            let fallback = match inherited {
                Some(Paint::RadialGradient {
                    center,
                    focal,
                    radius,
                    ..
                }) => Some((center, focal, radius)),
                _ => None,
            };
            let center = Point {
                x: if element.attr("cx").is_some() {
                    coordinate("cx", "50%", x_axis)?
                } else {
                    fallback.map_or(x_axis / 2.0, |v| v.0.x)
                },
                y: if element.attr("cy").is_some() {
                    coordinate("cy", "50%", y_axis)?
                } else {
                    fallback.map_or(y_axis / 2.0, |v| v.0.y)
                },
            };
            let focal = Point {
                x: if element.attr("fx").is_some() {
                    coordinate("fx", "50%", x_axis)?
                } else {
                    fallback.map_or(center.x, |v| v.1.x)
                },
                y: if element.attr("fy").is_some() {
                    coordinate("fy", "50%", y_axis)?
                } else {
                    fallback.map_or(center.y, |v| v.1.y)
                },
            };
            let radius = if element.attr("r").is_some() {
                coordinate("r", "50%", radius_axis)?
            } else {
                fallback.map_or(radius_axis / 2.0, |v| v.2)
            };
            Ok(Paint::RadialGradient {
                center,
                focal,
                radius,
                stops,
                spread,
                transform,
            })
        }
    }

    fn resolve_clip(
        &mut self,
        reference: &str,
        parent: Transform,
        viewport: Size,
    ) -> Result<crate::Clip, VectorDecodeError> {
        let index = *self
            .document
            .ids
            .get(local_reference(reference)?)
            .ok_or(VectorDecodeError::InvalidData)?;
        let element = &self.document.elements[index];
        if element.name != "clipPath" {
            return Err(VectorDecodeError::InvalidData);
        }
        if element.attr("clipPathUnits") == Some("objectBoundingBox") {
            return Err(VectorDecodeError::UnsupportedFeature(
                "object bounding box clip paths",
            ));
        }
        let transform = element
            .attr("transform")
            .map(parse_transform)
            .transpose()?
            .unwrap_or(Transform::IDENTITY)
            .then(parent);
        let mut path = Vec::new();
        for &child in &element.children {
            let child = &self.document.elements[child];
            if skips_unsupported_subtree(child.name) {
                continue;
            }
            let child_transform = match child.attr("transform").map(parse_transform).transpose() {
                Ok(transform) => transform.unwrap_or(Transform::IDENTITY),
                Err(VectorDecodeError::UnsupportedFeature(_)) => continue,
                Err(error) => return Err(error),
            };
            let child_path = match shape_path(child, viewport) {
                Ok(Some(path)) => path,
                Ok(None) | Err(VectorDecodeError::UnsupportedFeature(_)) => continue,
                Err(error) => return Err(error),
            };
            let mut child_path = child_path;
            for segment in &mut child_path {
                transform_segment(segment, child_transform);
            }
            path.extend(child_path);
        }
        if path.is_empty() {
            return Err(VectorDecodeError::UnsupportedFeature("clip path content"));
        }
        self.items = self
            .items
            .checked_add(path.len())
            .ok_or(VectorDecodeError::ResourceLimit)?;
        if self.items > MAX_ITEMS {
            return Err(VectorDecodeError::ResourceLimit);
        }
        let local_bounds = path_bounds(&path).ok_or(VectorDecodeError::InvalidData)?;
        let bounds = transform_bounds(local_bounds, transform);
        let path = self.image.add_path(path);
        Ok(crate::Clip {
            path,
            transform,
            rule: match element.attr("clip-rule") {
                Some("evenodd") => FillRule::EvenOdd,
                _ => FillRule::NonZero,
            },
            bounds,
        })
    }

    fn resolve_mask(
        &mut self,
        reference: &str,
        parent: Transform,
        object_bounds: Rect,
        viewport: Size,
    ) -> Result<Mask, VectorDecodeError> {
        let index = *self
            .document
            .ids
            .get(local_reference(reference)?)
            .ok_or(VectorDecodeError::InvalidData)?;
        let element = &self.document.elements[index];
        if element.name != "mask" || self.references.contains(&index) {
            return Err(VectorDecodeError::InvalidData);
        }
        let object_units = match element.attr("maskUnits").unwrap_or("objectBoundingBox") {
            "objectBoundingBox" => true,
            "userSpaceOnUse" => false,
            _ => return Err(VectorDecodeError::InvalidData),
        };
        let content_transform = match element.attr("maskContentUnits").unwrap_or("userSpaceOnUse") {
            "userSpaceOnUse" => parent,
            "objectBoundingBox" => Transform {
                a: object_bounds.width,
                d: object_bounds.height,
                e: object_bounds.x,
                f: object_bounds.y,
                ..Transform::IDENTITY
            }
            .then(parent),
            _ => return Err(VectorDecodeError::InvalidData),
        };
        let coordinate = |name: &str, default: f64, axis: f64, origin: f64| {
            let value = element.attr(name);
            if object_units {
                value
                    .map(object_bbox_length)
                    .transpose()
                    .map(|value| origin + value.unwrap_or(default) * axis)
            } else {
                value
                    .map(|value| length(value, axis))
                    .transpose()
                    .map(|value| value.unwrap_or(default * axis))
            }
        };
        let x_axis = if object_units {
            object_bounds.width
        } else {
            viewport.width
        };
        let y_axis = if object_units {
            object_bounds.height
        } else {
            viewport.height
        };
        let x_origin = if object_units { object_bounds.x } else { 0.0 };
        let y_origin = if object_units { object_bounds.y } else { 0.0 };
        let x = coordinate("x", -0.1, x_axis, x_origin)?;
        let y = coordinate("y", -0.1, y_axis, y_origin)?;
        let width = coordinate("width", 1.2, x_axis, 0.0)?;
        let height = coordinate("height", 1.2, y_axis, 0.0)?;
        if width < 0.0 || height < 0.0 {
            return Err(VectorDecodeError::InvalidData);
        }
        let local_region = Rect {
            x,
            y,
            width,
            height,
        };
        let bounds = transform_bounds(local_region, parent);
        let region_path = self
            .image
            .add_path(rect_path(x, y, width, height, 0.0, 0.0));
        let mask_style = parse_style(element, &Style::default())?;
        let start = self.image.command_len();
        self.references.push(index);
        let mask_viewport = if element.attr("maskContentUnits") == Some("objectBoundingBox") {
            Size {
                width: 1.0,
                height: 1.0,
            }
        } else {
            viewport
        };
        for &child in &element.children {
            if let Err(error) =
                self.render(child, &mask_style, content_transform, mask_viewport, false)
            {
                self.references.pop();
                return Err(error);
            }
        }
        self.references.pop();
        let commands = self.image.drain_commands(start);
        Ok(Mask {
            commands,
            region: crate::Clip {
                path: region_path,
                transform: parent,
                rule: FillRule::NonZero,
                bounds,
            },
            mask_type: parse_mask_type(element)?,
        })
    }
}
