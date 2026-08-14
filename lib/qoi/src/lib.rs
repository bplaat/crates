/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A small decoder for the Quite OK Image (QOI) format.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

const HEADER_LEN: usize = 14;
const END_MARKER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const MAX_PIXELS: u64 = 400_000_000;

/// A decoded QOI image with pixels in row-major RGBA order.
#[derive(Debug, Eq, PartialEq)]
pub struct Image {
    width: u32,
    height: u32,
    channels: u8,
    color_space: ColorSpace,
    pixels: Vec<u8>,
}

/// The color space declared by the QOI header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorSpace {
    /// sRGB color channels with a linear alpha channel.
    Srgb,
    /// Linear color and alpha channels.
    Linear,
}

impl Image {
    /// Returns the image width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the image height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the channel count declared by the QOI header.
    pub const fn channels(&self) -> u8 {
        self.channels
    }

    /// Returns the color space declared by the image.
    pub const fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// Returns the pixels in row-major RGBA order.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// An error produced while decoding a QOI image.
#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The input is not a QOI file.
    InvalidMagic,
    /// The header contains unsupported or invalid values.
    InvalidHeader,
    /// The encoded pixel stream ended unexpectedly or contains invalid data.
    InvalidData,
    /// The dimensions are too large for the current platform.
    ImageTooLarge,
}

impl Display for DecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidMagic => "not a QOI image",
            Self::InvalidHeader => "invalid QOI header",
            Self::InvalidData => "invalid QOI pixel data",
            Self::ImageTooLarge => "QOI image is too large",
        })
    }
}

impl Error for DecodeError {}

/// Decodes a complete QOI file to RGBA pixels.
pub fn decode(data: &[u8]) -> Result<Image, DecodeError> {
    if data.len() < HEADER_LEN + END_MARKER.len() {
        return Err(DecodeError::InvalidData);
    }
    if &data[..4] != b"qoif" {
        return Err(DecodeError::InvalidMagic);
    }

    let width = u32::from_be_bytes(data[4..8].try_into().expect("width has four bytes"));
    let height = u32::from_be_bytes(data[8..12].try_into().expect("height has four bytes"));
    if width == 0 || height == 0 || !matches!(data[12], 3 | 4) || data[13] > 1 {
        return Err(DecodeError::InvalidHeader);
    }

    let pixel_count = u64::from(width) * u64::from(height);
    if pixel_count >= MAX_PIXELS {
        return Err(DecodeError::ImageTooLarge);
    }
    let pixel_count = usize::try_from(pixel_count).map_err(|_| DecodeError::ImageTooLarge)?;
    let output_len = pixel_count
        .checked_mul(4)
        .ok_or(DecodeError::ImageTooLarge)?;
    let stream_end = data.len() - END_MARKER.len();
    if data[stream_end..] != END_MARKER {
        return Err(DecodeError::InvalidData);
    }
    if stream_end - HEADER_LEN < pixel_count.div_ceil(62) {
        return Err(DecodeError::InvalidData);
    }

    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(output_len)
        .map_err(|_| DecodeError::ImageTooLarge)?;
    let mut index = [[0u8; 4]; 64];
    let mut pixel = [0, 0, 0, 255];
    let mut cursor = HEADER_LEN;

    while pixels.len() < output_len {
        let byte = take(data, &mut cursor, stream_end)?;
        let run = match byte {
            0xfe => {
                pixel[0] = take(data, &mut cursor, stream_end)?;
                pixel[1] = take(data, &mut cursor, stream_end)?;
                pixel[2] = take(data, &mut cursor, stream_end)?;
                1
            }
            0xff => {
                pixel[0] = take(data, &mut cursor, stream_end)?;
                pixel[1] = take(data, &mut cursor, stream_end)?;
                pixel[2] = take(data, &mut cursor, stream_end)?;
                pixel[3] = take(data, &mut cursor, stream_end)?;
                1
            }
            _ if byte & 0xc0 == 0x00 => {
                pixel = index[usize::from(byte & 0x3f)];
                1
            }
            _ if byte & 0xc0 == 0x40 => {
                pixel[0] = pixel[0].wrapping_add((byte >> 4 & 0x03).wrapping_sub(2));
                pixel[1] = pixel[1].wrapping_add((byte >> 2 & 0x03).wrapping_sub(2));
                pixel[2] = pixel[2].wrapping_add((byte & 0x03).wrapping_sub(2));
                1
            }
            _ if byte & 0xc0 == 0x80 => {
                let second = take(data, &mut cursor, stream_end)?;
                let green = (byte & 0x3f).wrapping_sub(32);
                pixel[0] = pixel[0].wrapping_add(green.wrapping_add((second >> 4).wrapping_sub(8)));
                pixel[1] = pixel[1].wrapping_add(green);
                pixel[2] =
                    pixel[2].wrapping_add(green.wrapping_add((second & 0x0f).wrapping_sub(8)));
                1
            }
            _ => usize::from(byte & 0x3f) + 1,
        };

        let remaining = (output_len - pixels.len()) / 4;
        if run > remaining {
            return Err(DecodeError::InvalidData);
        }
        index[pixel_hash(pixel)] = pixel;
        for _ in 0..run {
            pixels.extend_from_slice(&pixel);
        }
    }

    if cursor != stream_end {
        return Err(DecodeError::InvalidData);
    }
    Ok(Image {
        width,
        height,
        channels: data[12],
        color_space: if data[13] == 0 {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        },
        pixels,
    })
}

fn take(data: &[u8], cursor: &mut usize, end: usize) -> Result<u8, DecodeError> {
    if *cursor >= end {
        return Err(DecodeError::InvalidData);
    }
    let byte = data[*cursor];
    *cursor += 1;
    Ok(byte)
}

const fn pixel_hash(pixel: [u8; 4]) -> usize {
    (pixel[0] as usize * 3 + pixel[1] as usize * 5 + pixel[2] as usize * 7 + pixel[3] as usize * 11)
        % 64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(width: u32, height: u32, channels: u8, chunks: &[u8]) -> Vec<u8> {
        let mut data = b"qoif".to_vec();
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&[channels, 0]);
        data.extend_from_slice(chunks);
        data.extend_from_slice(&END_MARKER);
        data
    }

    #[test]
    fn decodes_rgb_rgba_diff_luma_run_and_index() {
        let data = file(
            7,
            1,
            4,
            &[
                0xfe, 10, 20, 30, // RGB
                0xff, 1, 2, 3, 4,    // RGBA
                0x7f, // DIFF: +1, +1, +1
                0xa2, 0x79, // LUMA: dg=2, dr-dg=-1, db-dg=1
                0xc1, // RUN: repeat twice
                0x00, // INDEX: initial transparent black
            ],
        );
        let image = decode(&data).expect("image should decode");
        assert_eq!(image.width(), 7);
        assert_eq!(image.height(), 1);
        assert_eq!(image.channels(), 4);
        assert_eq!(image.color_space(), ColorSpace::Srgb);
        assert_eq!(
            image.pixels(),
            &[
                10, 20, 30, 255, 1, 2, 3, 4, 2, 3, 4, 4, 3, 5, 7, 4, 3, 5, 7, 4, 3, 5, 7, 4, 0, 0,
                0, 0,
            ]
        );
    }

    #[test]
    fn decodes_every_diff_value_with_wrapping() {
        for byte in 0x40..=0x7f {
            let image = decode(&file(1, 1, 4, &[byte])).expect("DIFF should decode");
            assert_eq!(
                image.pixels(),
                &[
                    ((byte >> 4 & 0x03) as i8 - 2) as u8,
                    ((byte >> 2 & 0x03) as i8 - 2) as u8,
                    ((byte & 0x03) as i8 - 2) as u8,
                    255,
                ],
                "DIFF byte {byte:#04x}"
            );
        }
    }

    #[test]
    fn decodes_every_luma_value_with_wrapping() {
        for first in 0x80..=0xbf {
            for second in 0..=u8::MAX {
                let green = i16::from(first & 0x3f) - 32;
                let red_green = i16::from(second >> 4) - 8;
                let blue_green = i16::from(second & 0x0f) - 8;
                let image = decode(&file(1, 1, 4, &[first, second])).expect("LUMA should decode");
                assert_eq!(
                    image.pixels(),
                    &[
                        (green + red_green) as u8,
                        green as u8,
                        (green + blue_green) as u8,
                        255,
                    ],
                    "LUMA bytes {first:#04x} {second:#04x}"
                );
            }
        }
    }

    #[test]
    fn decodes_every_valid_run_length() {
        for byte in 0xc0..=0xfd {
            let run = u32::from(byte & 0x3f) + 1;
            let image = decode(&file(run, 1, 3, &[byte])).expect("RUN should decode");
            assert_eq!(image.pixels().len(), run as usize * 4);
            assert!(
                image
                    .pixels()
                    .chunks_exact(4)
                    .all(|pixel| pixel == [0, 0, 0, 255])
            );
        }
    }

    #[test]
    fn rgb_preserves_alpha_and_index_restores_pixels() {
        let image = decode(&file(
            4,
            1,
            4,
            &[
                0xff,
                4,
                5,
                6,
                7, // RGBA
                0xfe,
                8,
                9,
                10, // RGB preserves alpha 7
                0xff,
                4,
                5,
                6,
                7,                               // Restore the first pixel
                pixel_hash([8, 9, 10, 7]) as u8, // INDEX restores the second pixel
            ],
        ))
        .expect("image should decode");
        assert_eq!(
            image.pixels(),
            &[4, 5, 6, 7, 8, 9, 10, 7, 4, 5, 6, 7, 8, 9, 10, 7]
        );
    }

    #[test]
    fn rejects_invalid_header_and_data() {
        assert_eq!(decode(b"not qoi"), Err(DecodeError::InvalidData));
        assert_eq!(decode(&file(0, 1, 4, &[])), Err(DecodeError::InvalidHeader));
        assert_eq!(decode(&file(1, 0, 4, &[])), Err(DecodeError::InvalidHeader));
        assert_eq!(
            decode(&file(1, 1, 2, &[0])),
            Err(DecodeError::InvalidHeader)
        );
        assert_eq!(
            decode(&file(1, 1, 5, &[0])),
            Err(DecodeError::InvalidHeader)
        );
        let mut invalid_color_space = file(1, 1, 4, &[0]);
        invalid_color_space[13] = 2;
        assert_eq!(
            decode(&invalid_color_space),
            Err(DecodeError::InvalidHeader)
        );
        assert_eq!(
            decode(&file(1, 1, 4, &[0xc1])),
            Err(DecodeError::InvalidData)
        );

        let mut trailing = file(1, 1, 4, &[0]);
        trailing.insert(15, 0);
        assert_eq!(decode(&trailing), Err(DecodeError::InvalidData));
    }

    #[test]
    fn rejects_truncated_chunks_and_bad_end_markers() {
        for chunks in [
            &[0xfe][..],
            &[0xfe, 1][..],
            &[0xfe, 1, 2][..],
            &[0xff][..],
            &[0xff, 1][..],
            &[0xff, 1, 2][..],
            &[0xff, 1, 2, 3][..],
            &[0x80][..],
        ] {
            assert_eq!(
                decode(&file(1, 1, 4, chunks)),
                Err(DecodeError::InvalidData)
            );
        }

        for marker_byte in 0..END_MARKER.len() {
            let mut data = file(1, 1, 4, &[0]);
            let index = data.len() - END_MARKER.len() + marker_byte;
            data[index] ^= 0xff;
            assert_eq!(decode(&data), Err(DecodeError::InvalidData));
        }
    }

    #[test]
    fn rejects_runs_past_the_image_and_excess_chunks() {
        assert_eq!(
            decode(&file(1, 1, 4, &[0xc1])),
            Err(DecodeError::InvalidData)
        );
        assert_eq!(
            decode(&file(1, 1, 4, &[0x00, 0x00])),
            Err(DecodeError::InvalidData)
        );
    }

    #[test]
    fn rejects_reference_pixel_limit_without_allocating() {
        assert_eq!(
            decode(&file(MAX_PIXELS as u32, 1, 4, &[])),
            Err(DecodeError::ImageTooLarge)
        );
        assert_eq!(
            decode(&file(u32::MAX, u32::MAX, 4, &[])),
            Err(DecodeError::ImageTooLarge)
        );
    }

    #[test]
    fn rejects_impossibly_short_large_image_without_allocating() {
        assert_eq!(
            decode(&file((MAX_PIXELS - 1) as u32, 1, 4, &[])),
            Err(DecodeError::InvalidData)
        );
    }

    #[test]
    fn accepts_both_channel_metadata_values() {
        for channels in [3, 4] {
            let image = decode(&file(1, 1, channels, &[0])).expect("image should decode");
            assert_eq!(image.channels(), channels);
        }
    }

    #[test]
    fn distinguishes_bad_magic() {
        let mut data = file(1, 1, 4, &[0]);
        data[0] = b'x';
        assert_eq!(decode(&data), Err(DecodeError::InvalidMagic));
    }

    #[test]
    fn preserves_linear_color_space() {
        let mut data = file(1, 1, 4, &[0]);
        data[13] = 1;
        assert_eq!(
            decode(&data).expect("image should decode").color_space(),
            ColorSpace::Linear
        );
    }
}
