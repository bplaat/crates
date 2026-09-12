/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Complete-file raster decoding to straight-alpha RGBA8 frames.
//!
//! Supports QOI, 8-bit sequential/progressive JPEG, PNG/APNG up to 8 bits per
//! channel, GIF, common BMP variants and ICO. Decoding is synchronous and does not
//! publish intermediate progressive scans. ICC profiles are not applied.
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

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
    /// A recognized format uses an unsupported feature, such as 16-bit PNG.
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

/// Decodes a complete encoded image, detecting its format from the bytes.
///
/// The cumulative allocation budget is 512 MiB, including intermediate buffers.
/// PNG/APNG samples above 8 bits and uncommon JPEG/BMP variants are unsupported.
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

#[cfg(any(
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp"
))]
const MAX_BYTES: usize = 512 * 1024 * 1024;

// Count cumulative allocations conservatively, including buffers later discarded.
#[derive(Default)]
#[cfg(any(
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp"
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

    #[cfg(any(feature = "jpeg", feature = "png", feature = "gif", feature = "bmp"))]
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
        let mut out = self.zeroed(bytes.len())?;
        out.copy_from_slice(bytes);
        Ok(out)
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    fn append<T: Copy>(&mut self, out: &mut Vec<T>, bytes: &[T]) -> Result<()> {
        self.claim(size_of_val(bytes))?;
        out.try_reserve_exact(bytes.len())
            .map_err(|_| DecodeError::ImageTooLarge)?;
        out.extend_from_slice(bytes);
        Ok(())
    }

    #[cfg(any(feature = "png", feature = "gif"))]
    fn frame(&mut self, frames: &mut Vec<Frame>, pixels: Vec<u8>, delay: Duration) -> Result<()> {
        self.claim(size_of::<Frame>())?;
        frames
            .try_reserve_exact(1)
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
}

#[cfg(all(
    test,
    feature = "qoi",
    feature = "jpeg",
    feature = "png",
    feature = "gif",
    feature = "bmp"
))]
mod tests {
    use super::*;

    const FIXTURES: &[(&str, &[u8], &[u8])] = &[
        (
            "rgb.png",
            include_bytes!("../tests/fixtures/rgb.png"),
            include_bytes!("../tests/fixtures/rgb.png.rgba"),
        ),
        (
            "rgba.png",
            include_bytes!("../tests/fixtures/rgba.png"),
            include_bytes!("../tests/fixtures/rgba.png.rgba"),
        ),
        (
            "adam7.png",
            include_bytes!("../tests/fixtures/adam7.png"),
            include_bytes!("../tests/fixtures/adam7.png.rgba"),
        ),
        (
            "palette.png",
            include_bytes!("../tests/fixtures/palette.png"),
            include_bytes!("../tests/fixtures/palette.png.rgba"),
        ),
        (
            "gray.png",
            include_bytes!("../tests/fixtures/gray.png"),
            include_bytes!("../tests/fixtures/gray.png.rgba"),
        ),
        (
            "rgb.qoi",
            include_bytes!("../tests/fixtures/rgb.qoi"),
            include_bytes!("../tests/fixtures/rgb.qoi.rgba"),
        ),
        (
            "rgba.qoi",
            include_bytes!("../tests/fixtures/rgba.qoi"),
            include_bytes!("../tests/fixtures/rgba.qoi.rgba"),
        ),
        (
            "operations.qoi",
            include_bytes!("../tests/fixtures/operations.qoi"),
            include_bytes!("../tests/fixtures/operations.qoi.rgba"),
        ),
        (
            "baseline.jpg",
            include_bytes!("../tests/fixtures/baseline.jpg"),
            include_bytes!("../tests/fixtures/baseline.jpg.rgba"),
        ),
        (
            "subsampled.jpg",
            include_bytes!("../tests/fixtures/subsampled.jpg"),
            include_bytes!("../tests/fixtures/subsampled.jpg.rgba"),
        ),
        (
            "progressive.jpg",
            include_bytes!("../tests/fixtures/progressive.jpg"),
            include_bytes!("../tests/fixtures/progressive.jpg.rgba"),
        ),
        (
            "gray.jpg",
            include_bytes!("../tests/fixtures/gray.jpg"),
            include_bytes!("../tests/fixtures/gray.jpg.rgba"),
        ),
        (
            "cmyk.jpg",
            include_bytes!("../tests/fixtures/cmyk.jpg"),
            include_bytes!("../tests/fixtures/cmyk.jpg.rgba"),
        ),
        (
            "rgb.bmp",
            include_bytes!("../tests/fixtures/rgb.bmp"),
            include_bytes!("../tests/fixtures/rgb.bmp.rgba"),
        ),
        (
            "palette.bmp",
            include_bytes!("../tests/fixtures/palette.bmp"),
            include_bytes!("../tests/fixtures/palette.bmp.rgba"),
        ),
        (
            "static.gif",
            include_bytes!("../tests/fixtures/static.gif"),
            include_bytes!("../tests/fixtures/static.gif.rgba"),
        ),
        (
            "interlaced.gif",
            include_bytes!("../tests/fixtures/interlaced.gif"),
            include_bytes!("../tests/fixtures/interlaced.gif.rgba"),
        ),
        (
            "restart.jpg",
            include_bytes!("../tests/fixtures/restart.jpg"),
            include_bytes!("../tests/fixtures/restart.jpg.rgba"),
        ),
        (
            "progressive-restart.jpg",
            include_bytes!("../tests/fixtures/progressive-restart.jpg"),
            include_bytes!("../tests/fixtures/progressive-restart.jpg.rgba"),
        ),
        (
            "separate.jpg",
            include_bytes!("../tests/fixtures/separate.jpg"),
            include_bytes!("../tests/fixtures/separate.jpg.rgba"),
        ),
        (
            "direct-rgb.jpg",
            include_bytes!("../tests/fixtures/direct-rgb.jpg"),
            include_bytes!("../tests/fixtures/direct-rgb.jpg.rgba"),
        ),
        (
            "horizontal.jpg",
            include_bytes!("../tests/fixtures/horizontal.jpg"),
            include_bytes!("../tests/fixtures/horizontal.jpg.rgba"),
        ),
        (
            "vertical.jpg",
            include_bytes!("../tests/fixtures/vertical.jpg"),
            include_bytes!("../tests/fixtures/vertical.jpg.rgba"),
        ),
    ];

    #[test]
    fn independent_reference_pixels() {
        for &(name, encoded, expected) in FIXTURES {
            let image = decode(encoded).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!((image.width(), image.height()), (19, 13), "{name}");
            assert_eq!(image.frames().len(), 1, "{name}");
            assert_eq!(image.pixels().len(), expected.len(), "{name}");
            if name.ends_with(".jpg") {
                // Independent IDCT and chroma interpolation rounding can differ slightly.
                let errors: Vec<_> = image
                    .pixels()
                    .iter()
                    .zip(expected)
                    .map(|(a, b)| a.abs_diff(*b) as usize)
                    .collect();
                let max = *errors.iter().max().expect("nonempty pixels");
                let mean = errors.iter().sum::<usize>() as f64 / errors.len() as f64;
                assert!(max <= 4 && mean < 1.0, "{name}: max {max}, mean {mean}");
            } else {
                assert_eq!(image.pixels(), expected, "{name}");
            }
        }
    }

    #[test]
    fn gif_animation_reference_and_timing() {
        let image =
            decode(include_bytes!("../tests/fixtures/animated.gif")).expect("GIF animation");
        let expected = include_bytes!("../tests/fixtures/animated.gif.rgba");
        assert_eq!(image.loop_count(), 3);
        assert_eq!(image.frames().len(), 2);
        for (index, frame) in image.frames().iter().enumerate() {
            assert_eq!(frame.delay(), Duration::from_millis([70, 130][index]));
            assert_eq!(
                frame.pixels(),
                &expected[index * 19 * 13 * 4..(index + 1) * 19 * 13 * 4]
            );
        }
    }

    #[test]
    fn every_truncation_is_rejected() {
        for &(name, bytes, _) in FIXTURES {
            for end in 0..bytes.len() {
                assert!(decode(&bytes[..end]).is_err(), "{name} at {end}");
            }
        }
    }

    #[test]
    fn corrupted_inputs_do_not_panic() {
        for &(_, bytes, _) in FIXTURES {
            for i in (0..bytes.len()).step_by(7) {
                let mut bad = bytes.to_vec();
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
}
