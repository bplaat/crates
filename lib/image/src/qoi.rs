/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{Budget, ColorSpace, DecodeError, Format, Image};

const HEADER_LEN: usize = 14;
const END_MARKER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const MAX_PIXELS: u64 = 400_000_000;

/// Decodes a complete QOI file to RGBA pixels.
pub(super) fn decode(data: &[u8]) -> Result<Image, DecodeError> {
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
    let mut budget = Budget::default();
    budget.claim(output_len)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(output_len)
        .map_err(|_| DecodeError::ImageTooLarge)?;
    pixels.resize(output_len, 0);
    let mut written = 0;
    let mut index = [[0u8; 4]; 64];
    let mut pixel = [0, 0, 0, 255];
    let mut cursor = HEADER_LEN;
    while written < output_len {
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
        let remaining = (output_len - written) / 4;
        if run > remaining {
            return Err(DecodeError::InvalidData);
        }
        index[pixel_hash(pixel)] = pixel;
        let end = written + run * 4;
        pixels[written..end].as_chunks_mut::<4>().0.fill(pixel);
        written = end;
    }
    if cursor != stream_end {
        return Err(DecodeError::InvalidData);
    }
    let mut image = Image::still(Format::Qoi, width, height, pixels);
    image.color_space = if data[13] == 0 {
        ColorSpace::Srgb
    } else {
        ColorSpace::Linear
    };
    Ok(image)
}

const fn take(data: &[u8], cursor: &mut usize, end: usize) -> Result<u8, DecodeError> {
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

    #[test]
    fn operation_stream_matches_reference_pixels() {
        let image = decode(include_bytes!("../tests/fixtures/operations.qoi"))
            .expect("QOI operation fixture");
        assert_eq!(
            image.pixels(),
            include_bytes!("../tests/fixtures/operations.qoi.rgba")
        );
    }
}
