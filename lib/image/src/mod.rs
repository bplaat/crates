/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[cfg(all(
    test,
    any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp",
        feature = "ico",
        feature = "svg",
        feature = "tinyvg"
    )
))]
mod test_support;
mod vector;
pub use vector::{
    Clip, Color, DrawCommand, FillRule, GradientStop, LineCap, LineJoin, Mask, MaskType, Paint,
    PaintId, PathId, PathSegment, Point, Rect, Size, SpreadMethod, StrokeStyle, Transform,
    VectorColorSpace, VectorDecodeError, VectorFormat, VectorImage,
};

#[cfg(feature = "svg")]
mod svg;
#[cfg(feature = "tinyvg")]
mod tinyvg;

#[cfg(feature = "bmp")]
mod bmp;
#[cfg(feature = "gif")]
mod gif;
#[cfg(feature = "ico")]
mod ico;
#[cfg(feature = "jpeg")]
mod jpeg;
#[cfg(feature = "png")]
mod png;
#[cfg(feature = "qoi")]
mod qoi;

/// The encoded format detected from an image's file signature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// Quite OK Image.
    #[cfg(feature = "qoi")]
    Qoi,
    /// JPEG.
    #[cfg(feature = "jpeg")]
    Jpeg,
    /// PNG, including APNG animations.
    #[cfg(feature = "png")]
    Png,
    /// Graphics Interchange Format.
    #[cfg(feature = "gif")]
    Gif,
    /// Windows or OS/2 bitmap.
    #[cfg(feature = "bmp")]
    Bmp,
    /// Windows icon.
    #[cfg(feature = "ico")]
    Ico,
}

/// The interpretation of decoded color channels. Alpha is always linear.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorSpace {
    /// sRGB (also the default for images without supported color metadata).
    Srgb,
    /// Linear sRGB, as declared by QOI.
    Linear,
}

/// A complete displayed frame, after animation blending and before disposal.
#[derive(Debug, Eq, PartialEq)]
pub struct Frame {
    pixels: Vec<u8>,
    delay: Duration,
}

impl Frame {
    /// Returns canvas-sized, row-major, straight-alpha RGBA8 pixels.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns the encoded display duration; static images have zero duration.
    pub const fn delay(&self) -> Duration {
        self.delay
    }
}

/// A decoded image. JPEG orientation is already applied to dimensions and pixels.
#[derive(Debug, Eq, PartialEq)]
pub struct Image {
    format: Format,
    width: u32,
    height: u32,
    color_space: ColorSpace,
    frames: Vec<Frame>,
    loop_count: u32,
}

impl Image {
    /// Returns the encoded format.
    pub const fn format(&self) -> Format {
        self.format
    }

    /// Returns the displayed canvas width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the displayed canvas height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the interpretation of the decoded color channels.
    pub const fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// Returns one or more fully composited frames.
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    /// Returns the first displayed frame's RGBA8 pixels.
    pub fn pixels(&self) -> &[u8] {
        &self.frames[0].pixels
    }

    /// Returns total animation plays, or zero for infinite playback.
    pub const fn loop_count(&self) -> u32 {
        self.loop_count
    }

    #[cfg(any(feature = "qoi", feature = "jpeg", feature = "png", feature = "bmp"))]
    fn still(format: Format, width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            format,
            width,
            height,
            color_space: ColorSpace::Srgb,
            frames: vec![Frame {
                pixels,
                delay: Duration::ZERO,
            }],
            loop_count: 1,
        }
    }
}

/// A decoding failure. Invalid input never intentionally panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// No supported file signature was found.
    InvalidMagic,
    /// Invalid image dimensions or header fields.
    InvalidHeader,
    /// Truncated, corrupt or inconsistent encoded data.
    InvalidData,
    /// A recognized format uses an unsupported feature.
    UnsupportedFeature,
    /// Decoded storage exceeds 512 MiB, arithmetic overflows, or allocation fails.
    ImageTooLarge,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMagic => "unsupported image format",
            Self::InvalidHeader => "invalid image header",
            Self::InvalidData => "invalid or truncated image data",
            Self::UnsupportedFeature => "unsupported image feature",
            Self::ImageTooLarge => "image exceeds decoding resource limits",
        })
    }
}

impl std::error::Error for DecodeError {}

type Result<T> = std::result::Result<T, DecodeError>;

/// Decodes a complete image within 512 MiB, excluding uncommon JPEG/BMP variants.
#[cfg_attr(
    not(any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp"
    )),
    allow(clippy::missing_const_for_fn)
)]
pub fn decode(_data: &[u8]) -> Result<Image> {
    #[cfg(feature = "qoi")]
    if _data.starts_with(b"qoif") {
        return qoi::decode(_data);
    }
    #[cfg(feature = "jpeg")]
    if _data.starts_with(b"\xff\xd8") {
        return jpeg::decode(_data);
    }
    #[cfg(feature = "png")]
    if _data.starts_with(b"\x89PNG\r\n\x1a\n") {
        return png::decode(_data);
    }
    #[cfg(feature = "gif")]
    if _data.starts_with(b"GIF87a") || _data.starts_with(b"GIF89a") {
        return gif::decode(_data);
    }
    #[cfg(feature = "bmp")]
    if _data.starts_with(b"BM") {
        return bmp::decode(_data);
    }
    #[cfg(feature = "ico")]
    if _data.starts_with(b"\0\0\x01\0") {
        return ico::decode(_data);
    }
    Err(DecodeError::InvalidMagic)
}

/// Detects a supported vector format without fully decoding it.
#[cfg_attr(
    not(any(feature = "tinyvg", feature = "svg")),
    allow(clippy::missing_const_for_fn)
)]
pub fn vector_format(_data: &[u8]) -> Option<VectorFormat> {
    #[cfg(feature = "tinyvg")]
    if tinyvg::is_tinyvg(_data) {
        return Some(VectorFormat::TinyVg);
    }
    #[cfg(feature = "svg")]
    if svg::is_svg(_data) {
        return Some(VectorFormat::Svg);
    }
    None
}

/// Decodes a supported vector image to an immutable backend-neutral display list.
pub fn decode_vector(data: &[u8]) -> std::result::Result<VectorImage, VectorDecodeError> {
    if data.len() > 64 * 1024 * 1024 {
        return Err(VectorDecodeError::ResourceLimit);
    }
    match vector_format(data) {
        #[cfg(feature = "tinyvg")]
        Some(VectorFormat::TinyVg) => vector::decode_tinyvg(data),
        #[cfg(feature = "svg")]
        Some(VectorFormat::Svg) => svg::decode(data),
        _ => Err(VectorDecodeError::InvalidMagic),
    }
}

#[cfg(any(
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp",
    feature = "ico"
))]
const MAX_BYTES: usize = 512 * 1024 * 1024;

// Count cumulative allocations conservatively, including buffers later discarded.
#[derive(Default)]
#[cfg(any(
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp",
    feature = "ico"
))]
struct Budget {
    used: usize,
}

#[cfg(any(
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp"
))]
impl Budget {
    fn claim(&mut self, bytes: usize) -> Result<()> {
        self.used = self
            .used
            .checked_add(bytes)
            .filter(|n| *n <= MAX_BYTES)
            .ok_or(DecodeError::ImageTooLarge)?;
        Ok(())
    }

    #[cfg(any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp"
    ))]
    fn zeroed<T: Default + Clone>(&mut self, len: usize) -> Result<Vec<T>> {
        self.claim(
            len.checked_mul(size_of::<T>())
                .ok_or(DecodeError::ImageTooLarge)?,
        )?;
        let mut out = Vec::new();
        out.try_reserve_exact(len)
            .map_err(|_| DecodeError::ImageTooLarge)?;
        out.resize(len, T::default());
        Ok(out)
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    fn copy(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
        self.claim(bytes.len())?;
        let mut out = Vec::new();
        out.try_reserve_exact(bytes.len())
            .map_err(|_| DecodeError::ImageTooLarge)?;
        out.extend_from_slice(bytes);
        Ok(out)
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    fn append<T: Copy>(&mut self, out: &mut Vec<T>, bytes: &[T]) -> Result<()> {
        self.claim(size_of_val(bytes))?;
        out.try_reserve(bytes.len())
            .map_err(|_| DecodeError::ImageTooLarge)?;
        out.extend_from_slice(bytes);
        Ok(())
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    fn frame(&mut self, frames: &mut Vec<Frame>, pixels: Vec<u8>, delay: Duration) -> Result<()> {
        self.claim(size_of::<Frame>())?;
        frames
            .try_reserve(1)
            .map_err(|_| DecodeError::ImageTooLarge)?;
        frames.push(Frame { pixels, delay });
        Ok(())
    }
}

#[cfg(any(feature = "jpeg", feature = "png", feature = "gif", feature = "bmp"))]
fn pixel_len(width: u32, height: u32) -> Result<usize> {
    if width == 0 || height == 0 {
        return Err(DecodeError::InvalidHeader);
    }
    usize::try_from(
        (u64::from(width) * u64::from(height))
            .checked_mul(4)
            .ok_or(DecodeError::ImageTooLarge)?,
    )
    .ok()
    .filter(|n| *n <= MAX_BYTES)
    .ok_or(DecodeError::ImageTooLarge)
}

#[cfg(any(feature = "jpeg", feature = "png", feature = "gif", feature = "bmp"))]
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[cfg(any(feature = "jpeg", feature = "png", feature = "gif", feature = "bmp"))]
impl<'a> Reader<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(len).ok_or(DecodeError::InvalidData)?;
        let bytes = self
            .data
            .get(self.pos..end)
            .ok_or(DecodeError::InvalidData)?;
        self.pos = end;
        Ok(bytes)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    #[cfg(any(feature = "jpeg", feature = "png"))]
    fn be16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }

    #[cfg(feature = "png")]
    fn be32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    #[cfg(any(feature = "gif", feature = "bmp"))]
    fn le16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }

    #[cfg(feature = "bmp")]
    fn le32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    #[cfg(any(feature = "jpeg", feature = "png", feature = "bmp"))]
    const fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
}

#[derive(Clone, Copy)]
#[cfg(any(feature = "png", feature = "gif"))]
struct Area {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

#[cfg(any(feature = "png", feature = "gif"))]
impl Area {
    fn validate(self, width: u32, height: u32) -> Result<Self> {
        if self.width == 0
            || self.height == 0
            || self
                .x
                .checked_add(self.width)
                .is_none_or(|n| n > width as usize)
            || self
                .y
                .checked_add(self.height)
                .is_none_or(|n| n > height as usize)
        {
            return Err(DecodeError::InvalidData);
        }
        Ok(self)
    }

    fn clear(self, canvas: &mut [u8], stride: usize, pixel: [u8; 4]) {
        for y in self.y..self.y + self.height {
            canvas[(y * stride + self.x) * 4..(y * stride + self.x + self.width) * 4]
                .as_chunks_mut::<4>()
                .0
                .fill(pixel);
        }
    }

    fn snapshot(self, canvas: &[u8], stride: usize, budget: &mut Budget) -> Result<Vec<u8>> {
        let row_len = self
            .width
            .checked_mul(4)
            .ok_or(DecodeError::ImageTooLarge)?;
        let len = row_len
            .checked_mul(self.height)
            .ok_or(DecodeError::ImageTooLarge)?;
        budget.claim(len)?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(len)
            .map_err(|_| DecodeError::ImageTooLarge)?;
        for row in 0..self.height {
            let start = ((self.y + row) * stride + self.x) * 4;
            pixels.extend_from_slice(&canvas[start..start + row_len]);
        }
        Ok(pixels)
    }

    fn restore(self, canvas: &mut [u8], stride: usize, pixels: &[u8]) {
        let row_len = self.width * 4;
        for (row, source) in pixels.chunks_exact(row_len).enumerate() {
            let start = ((self.y + row) * stride + self.x) * 4;
            canvas[start..start + row_len].copy_from_slice(source);
        }
    }
}

#[cfg(all(
    test,
    any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp",
        feature = "ico"
    )
))]
mod tests {
    use super::*;

    #[cfg(feature = "qoi")]
    #[test]
    fn qoi_reference_corpus() {
        test_support::run_raster_format("qoi");
    }

    #[cfg(feature = "jpeg")]
    #[test]
    fn jpeg_reference_corpus() {
        test_support::run_raster_format("jpeg");
    }

    #[cfg(feature = "png")]
    #[test]
    fn png_reference_corpus() {
        test_support::run_raster_format("png");
    }

    #[cfg(feature = "gif")]
    #[test]
    fn gif_reference_corpus() {
        test_support::run_raster_format("gif");
    }

    #[cfg(feature = "bmp")]
    #[test]
    fn bmp_reference_corpus() {
        test_support::run_raster_format("bmp");
    }

    #[cfg(feature = "ico")]
    #[test]
    fn ico_reference_corpus() {
        test_support::run_raster_format("ico");
    }
    #[cfg(any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp",
        feature = "ico"
    ))]
    #[test]
    fn every_truncation_is_rejected() {
        for (name, encoded) in test_support::raster_inputs() {
            for end in 0..encoded.len().min(32) {
                assert!(
                    decode(&encoded[..end]).is_err(),
                    "{} at {end}",
                    name.to_string_lossy()
                );
            }
        }
    }

    #[cfg(any(
        feature = "qoi",
        feature = "jpeg",
        feature = "png",
        feature = "gif",
        feature = "bmp",
        feature = "ico"
    ))]
    #[test]
    fn corrupted_inputs_do_not_panic() {
        for (_, encoded) in test_support::raster_inputs() {
            for i in (0..encoded.len()).step_by(7) {
                let mut bad = encoded.clone();
                bad[i] ^= 0xff;
                let _ = decode(&bad);
            }
        }
        let mut state = 1234567u32;
        for len in 0..512 {
            let mut bytes = vec![0; len];
            for b in &mut bytes {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                *b = state as u8;
            }
            for signature in [
                &b"\xff\xd8"[..],
                b"GIF89a",
                b"BM",
                b"\0\0\x01\0",
                b"qoif",
                b"\x89PNG\r\n\x1a\n",
            ] {
                let mut input = signature.to_vec();
                input.extend_from_slice(&bytes);
                let _ = decode(&input);
            }
        }
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    #[test]
    fn animation_snapshot_only_stores_the_changed_area() {
        let area = Area {
            x: 1,
            y: 1,
            width: 1,
            height: 1,
        };
        let mut canvas = (0..24).collect::<Vec<_>>();
        let original = canvas.clone();
        let mut budget = Budget::default();
        let snapshot = area
            .snapshot(&canvas, 3, &mut budget)
            .expect("area snapshot");
        assert_eq!(snapshot.len(), 4);
        canvas[16..20].fill(255);
        area.restore(&mut canvas, 3, &snapshot);
        assert_eq!(canvas, original);
    }
}
