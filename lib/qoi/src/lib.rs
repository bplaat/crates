/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [qoi](https://crates.io/crates/qoi) crate.
//! Implements QOI (Quite OK Image) format decoding.

use std::fmt;

// MARK: Types

/// Image channel count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channels {
    /// 3 bytes per pixel: red, green, blue.
    Rgb = 3,
    /// 4 bytes per pixel: red, green, blue, alpha.
    Rgba = 4,
}

/// Image color space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    /// sRGB with linear alpha channel.
    Srgb = 0,
    /// All channels linear.
    Linear = 1,
}

/// Decoded image header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Number of color channels.
    pub channels: Channels,
    /// Color space of the image data.
    pub colorspace: ColorSpace,
}

/// Error type for QOI decode failures.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "qoi error: {}", self.0)
    }
}

impl std::error::Error for Error {}

/// Result type for QOI decode operations.
pub type Result<T> = std::result::Result<T, Error>;

// MARK: Decode

fn color_hash(r: u8, g: u8, b: u8, a: u8) -> usize {
    (r as usize * 3 + g as usize * 5 + b as usize * 7 + a as usize * 11) % 64
}

/// Decode a QOI-encoded byte slice into raw pixel data.
///
/// Returns a [`Header`] describing the image and a `Vec<u8>` with
/// `width * height * channels` bytes of pixel data in row-major order.
pub fn decode_to_vec(data: impl AsRef<[u8]>) -> Result<(Header, Vec<u8>)> {
    let data = data.as_ref();
    if data.len() < 14 {
        return Err(Error("data too short".into()));
    }
    if &data[0..4] != b"qoif" {
        return Err(Error("invalid magic bytes".into()));
    }
    let width = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let height = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let channels = match data[12] {
        3 => Channels::Rgb,
        4 => Channels::Rgba,
        c => return Err(Error(format!("invalid channels value: {c}"))),
    };
    let colorspace = match data[13] {
        0 => ColorSpace::Srgb,
        1 => ColorSpace::Linear,
        c => return Err(Error(format!("invalid colorspace value: {c}"))),
    };

    let header = Header {
        width,
        height,
        channels,
        colorspace,
    };
    let pixel_count = width as usize * height as usize;
    let ch = channels as usize;
    let mut out = vec![0u8; pixel_count * ch];

    let mut running = [[0u8; 4]; 64];
    let mut prev = [0u8, 0, 0, 255];
    let mut run = 0usize;
    let mut i = 14usize;

    for px_idx in 0..pixel_count {
        let pixel = if run > 0 {
            run -= 1;
            prev
        } else {
            if i >= data.len() {
                return Err(Error("unexpected end of stream".into()));
            }
            let byte = data[i];
            i += 1;

            let px = if byte == 0xFE {
                if i + 3 > data.len() {
                    return Err(Error("truncated RGB op".into()));
                }
                let p = [data[i], data[i + 1], data[i + 2], prev[3]];
                i += 3;
                p
            } else if byte == 0xFF {
                if i + 4 > data.len() {
                    return Err(Error("truncated RGBA op".into()));
                }
                let p = [data[i], data[i + 1], data[i + 2], data[i + 3]];
                i += 4;
                p
            } else {
                match byte >> 6 {
                    0b00 => running[(byte & 0x3F) as usize],
                    0b01 => {
                        let dr = ((byte >> 4) & 0x3).wrapping_sub(2);
                        let dg = ((byte >> 2) & 0x3).wrapping_sub(2);
                        let db = (byte & 0x3).wrapping_sub(2);
                        [
                            prev[0].wrapping_add(dr),
                            prev[1].wrapping_add(dg),
                            prev[2].wrapping_add(db),
                            prev[3],
                        ]
                    }
                    0b10 => {
                        if i >= data.len() {
                            return Err(Error("truncated LUMA op".into()));
                        }
                        let b2 = data[i];
                        i += 1;
                        let dg = (byte & 0x3F).wrapping_sub(32);
                        let dr = ((b2 >> 4) & 0xF).wrapping_sub(8).wrapping_add(dg);
                        let db = (b2 & 0xF).wrapping_sub(8).wrapping_add(dg);
                        [
                            prev[0].wrapping_add(dr),
                            prev[1].wrapping_add(dg),
                            prev[2].wrapping_add(db),
                            prev[3],
                        ]
                    }
                    _ => {
                        run = (byte & 0x3F) as usize;
                        prev
                    }
                }
            };

            running[color_hash(px[0], px[1], px[2], px[3])] = px;
            prev = px;
            px
        };

        let p = px_idx * ch;
        out[p] = pixel[0];
        out[p + 1] = pixel[1];
        out[p + 2] = pixel[2];
        if ch == 4 {
            out[p + 3] = pixel[3];
        }
    }

    Ok((header, out))
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn make_qoi_header(width: u32, height: u32, channels: u8, colorspace: u8) -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(b"qoif");
        h.extend_from_slice(&width.to_be_bytes());
        h.extend_from_slice(&height.to_be_bytes());
        h.push(channels);
        h.push(colorspace);
        h
    }

    #[test]
    fn test_decode_header_rgb() {
        let mut data = make_qoi_header(4, 2, 3, 0);
        // 8 solid red pixels via RUN op (2x4=8 pixels, run value 7 = 8 pixels)
        data.push(0xFE);
        data.push(255);
        data.push(0);
        data.push(0);
        data.push(0xC0 | 6); // run of 7 more (prev is already counted)
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
        let (header, pixels) = decode_to_vec(&data).expect("decode failed");
        assert_eq!(header.width, 4);
        assert_eq!(header.height, 2);
        assert_eq!(header.channels, Channels::Rgb);
        assert_eq!(header.colorspace, ColorSpace::Srgb);
        assert_eq!(pixels.len(), 4 * 2 * 3);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"INVALID_HEADER_DATA_HERE";
        assert!(decode_to_vec(data).is_err());
    }

    #[test]
    fn test_data_too_short() {
        assert!(decode_to_vec(b"qoi").is_err());
    }

    #[test]
    fn test_invalid_channels() {
        let mut data = make_qoi_header(1, 1, 5, 0);
        data.extend_from_slice(&[0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert!(decode_to_vec(&data).is_err());
    }
}
