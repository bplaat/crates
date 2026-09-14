/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{Budget, ColorSpace, DecodeError, Format, Image};

const HEADER_LEN: usize = 14;
const END_MARKER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const MAX_PIXELS: u64 = 400_000_000;

struct Decoder<'a> {
    data: &'a [u8],
    cursor: usize,
    stream_end: usize,
    index: [[u8; 4]; 64],
    pixel: [u8; 4],
}

/// Decodes a complete QOI file to RGBA pixels.
pub(super) fn decode(data: &[u8]) -> Result<Image, DecodeError> {
    Decoder::new(data)?.decode()
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Result<Self, DecodeError> {
        if data.len() < HEADER_LEN + END_MARKER.len() {
            return Err(DecodeError::InvalidData);
        }
        if &data[..4] != b"qoif" {
            return Err(DecodeError::InvalidMagic);
        }
        let stream_end = data.len() - END_MARKER.len();
        if data[stream_end..] != END_MARKER {
            return Err(DecodeError::InvalidData);
        }
        Ok(Self {
            data,
            cursor: HEADER_LEN,
            stream_end,
            index: [[0; 4]; 64],
            pixel: [0, 0, 0, 255],
        })
    }

    fn decode(mut self) -> Result<Image, DecodeError> {
        let width = u32::from_be_bytes(self.data[4..8].try_into().expect("four bytes"));
        let height = u32::from_be_bytes(self.data[8..12].try_into().expect("four bytes"));
        if width == 0 || height == 0 || !matches!(self.data[12], 3 | 4) || self.data[13] > 1 {
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
        if self.stream_end - HEADER_LEN < pixel_count.div_ceil(62) {
            return Err(DecodeError::InvalidData);
        }
        let mut budget = Budget::default();
        let mut pixels = budget.zeroed(output_len)?;
        let mut written = 0;
        while written < output_len {
            let byte = self.take()?;
            let run = match byte {
                0xfe => {
                    self.pixel[0] = self.take()?;
                    self.pixel[1] = self.take()?;
                    self.pixel[2] = self.take()?;
                    1
                }
                0xff => {
                    self.pixel[0] = self.take()?;
                    self.pixel[1] = self.take()?;
                    self.pixel[2] = self.take()?;
                    self.pixel[3] = self.take()?;
                    1
                }
                _ if byte & 0xc0 == 0x00 => {
                    self.pixel = self.index[usize::from(byte & 0x3f)];
                    1
                }
                _ if byte & 0xc0 == 0x40 => {
                    self.pixel[0] = self.pixel[0].wrapping_add((byte >> 4 & 3).wrapping_sub(2));
                    self.pixel[1] = self.pixel[1].wrapping_add((byte >> 2 & 3).wrapping_sub(2));
                    self.pixel[2] = self.pixel[2].wrapping_add((byte & 3).wrapping_sub(2));
                    1
                }
                _ if byte & 0xc0 == 0x80 => {
                    let second = self.take()?;
                    let green = (byte & 0x3f).wrapping_sub(32);
                    self.pixel[0] = self.pixel[0]
                        .wrapping_add(green.wrapping_add((second >> 4).wrapping_sub(8)));
                    self.pixel[1] = self.pixel[1].wrapping_add(green);
                    self.pixel[2] = self.pixel[2]
                        .wrapping_add(green.wrapping_add((second & 15).wrapping_sub(8)));
                    1
                }
                _ => usize::from(byte & 0x3f) + 1,
            };
            let remaining = (output_len - written) / 4;
            if run > remaining {
                return Err(DecodeError::InvalidData);
            }
            self.index[Self::pixel_hash(self.pixel)] = self.pixel;
            let end = written + run * 4;
            pixels[written..end].as_chunks_mut::<4>().0.fill(self.pixel);
            written = end;
        }
        if self.cursor != self.stream_end {
            return Err(DecodeError::InvalidData);
        }
        let mut image = Image::still(Format::Qoi, width, height, pixels);
        image.color_space = if self.data[13] == 0 {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        };
        Ok(image)
    }

    const fn take(&mut self) -> Result<u8, DecodeError> {
        if self.cursor >= self.stream_end {
            return Err(DecodeError::InvalidData);
        }
        let byte = self.data[self.cursor];
        self.cursor += 1;
        Ok(byte)
    }

    const fn pixel_hash(pixel: [u8; 4]) -> usize {
        (pixel[0] as usize * 3
            + pixel[1] as usize * 5
            + pixel[2] as usize * 7
            + pixel[3] as usize * 11)
            % 64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_rs_basic_fixture_decodes() {
        let image = decode(include_bytes!("../tests/images/qoi/basic-test.qoi"))
            .expect("image-rs QOI fixture");
        assert_eq!((image.width(), image.height()), (5, 5));
        assert_eq!(image.pixels().len(), 5 * 5 * 4);
    }
}
