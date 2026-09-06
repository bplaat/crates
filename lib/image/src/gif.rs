/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::mem;
use std::time::Duration;

use super::{Area, Budget, ColorSpace, DecodeError, Format, Image, Reader, Result, pixel_len};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum DisposalMethod {
    #[default]
    None,
    Keep,
    Background,
    Previous,
}

impl TryFrom<u8> for DisposalMethod {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Keep),
            2 => Ok(Self::Background),
            3 => Ok(Self::Previous),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

#[derive(Default)]
struct GraphicControl {
    delay: Duration,
    transparent_index: Option<u8>,
    disposal: DisposalMethod,
}

pub(super) fn decode(data: &[u8]) -> Result<Image> {
    // Extension blocks configure the next frame. Image descriptors decompress palette indexes,
    // composite a displayed frame, and then apply its disposal operation.
    let mut r = Reader::new(&data[6..]);
    let width = u32::from(r.le16()?);
    let height = u32::from(r.le16()?);
    let len = pixel_len(width, height)?;
    let packed = r.byte()?;
    let background = r.byte()? as usize;
    r.byte()?; // Pixel aspect ratio does not change the pixel canvas.
    let mut global = [[0; 4]; 256];
    let global_len = table(&mut r, packed, &mut global)?;
    let mut budget = Budget::default();
    let mut canvas = Vec::new();
    let mut frames = Vec::new();
    let mut loop_count = 1;
    let mut control = GraphicControl::default();
    loop {
        match r.byte()? {
            0x21 => match r.byte()? {
                0xf9 => {
                    if r.byte()? != 4 {
                        return Err(DecodeError::InvalidData);
                    }
                    let flags = r.byte()?;
                    if flags & 0xe0 != 0 {
                        return Err(DecodeError::InvalidData);
                    }
                    control.disposal = DisposalMethod::try_from((flags >> 2) & 7)?;
                    control.delay = Duration::from_millis(u64::from(r.le16()?) * 10);
                    let index = r.byte()?;
                    control.transparent_index = if flags & 1 != 0 { Some(index) } else { None };
                    if r.byte()? != 0 {
                        return Err(DecodeError::InvalidData);
                    }
                }
                0xff => {
                    let size = r.byte()? as usize;
                    if size != 11 {
                        return Err(DecodeError::InvalidData);
                    }
                    let name = r.take(size)?;
                    let bytes = blocks(&mut r, &mut budget)?;
                    if matches!(name, b"NETSCAPE2.0" | b"ANIMEXTS1.0") {
                        if bytes.len() != 3 || bytes[0] != 1 {
                            return Err(DecodeError::InvalidData);
                        }
                        let repeats = u16::from_le_bytes([bytes[1], bytes[2]]);
                        loop_count = if repeats == 0 {
                            0
                        } else {
                            u32::from(repeats) + 1
                        };
                    }
                }
                0x01 => return Err(DecodeError::UnsupportedFeature), // Plain-text rendering.
                _ => {
                    skip_blocks(&mut r)?;
                }
            },
            0x2c => {
                let control = mem::take(&mut control);
                let x = r.le16()? as usize;
                let y = r.le16()? as usize;
                let w = r.le16()? as usize;
                let h = r.le16()? as usize;
                let area = Area {
                    x,
                    y,
                    width: w,
                    height: h,
                }
                .validate(width, height)?;
                let flags = r.byte()?;
                if flags & 0x18 != 0 {
                    return Err(DecodeError::InvalidData);
                }
                let mut local = [[0; 4]; 256];
                let local_len = table(&mut r, flags, &mut local)?;
                let palette = if local_len != 0 {
                    &local[..local_len]
                } else {
                    &global[..global_len]
                };
                if palette.is_empty()
                    || control
                        .transparent_index
                        .is_some_and(|t| t as usize >= palette.len())
                {
                    return Err(DecodeError::InvalidData);
                }
                let bg = if control.transparent_index.is_some() || global_len == 0 {
                    [0; 4]
                } else {
                    *global
                        .get(background)
                        .filter(|_| background < global_len)
                        .ok_or(DecodeError::InvalidData)?
                };
                let min_code = r.byte()?;
                let compressed = blocks(&mut r, &mut budget)?;
                let indices = lzw(&compressed, min_code, w * h, &mut budget)?;
                if canvas.is_empty() {
                    canvas = budget.zeroed(len)?;
                    canvas.as_chunks_mut::<4>().0.fill(bg);
                }
                let previous = if control.disposal == DisposalMethod::Previous {
                    Some(budget.copy(&canvas)?)
                } else {
                    None
                };
                let passes = if flags & 0x40 != 0 {
                    &[(0, 8), (4, 8), (2, 4), (1, 2)][..]
                } else {
                    &[(0, 1)][..]
                };
                let mut source_row = 0;
                for &(start, step) in passes {
                    for row in (start..h).step_by(step) {
                        let dest = ((y + row) * width as usize + x) * 4;
                        for (p, index) in canvas[dest..dest + w * 4]
                            .as_chunks_mut::<4>()
                            .0
                            .iter_mut()
                            .zip(&indices[source_row * w..(source_row + 1) * w])
                        {
                            let color = palette
                                .get(*index as usize)
                                .ok_or(DecodeError::InvalidData)?;
                            if Some(*index) != control.transparent_index {
                                p.copy_from_slice(color);
                            }
                        }
                        source_row += 1;
                    }
                }
                let pixels = budget.copy(&canvas)?;
                budget.frame(&mut frames, pixels, control.delay)?;
                match control.disposal {
                    DisposalMethod::Background => area.clear(&mut canvas, width as usize, bg),
                    DisposalMethod::Previous => {
                        canvas.copy_from_slice(&previous.expect("previous canvas was saved"));
                    }
                    _ => {}
                }
            }
            0x3b => {
                if frames.is_empty() {
                    return Err(DecodeError::InvalidData);
                }
                return Ok(Image {
                    format: Format::Gif,
                    width,
                    height,
                    color_space: ColorSpace::Srgb,
                    frames,
                    loop_count,
                });
            }
            _ => return Err(DecodeError::InvalidData),
        }
    }
}

fn table(r: &mut Reader<'_>, flags: u8, palette: &mut [[u8; 4]; 256]) -> Result<usize> {
    if flags & 0x80 == 0 {
        return Ok(0);
    }
    let count = 2usize << (flags & 7);
    for color in &mut palette[..count] {
        color[..3].copy_from_slice(r.take(3)?);
        color[3] = 255;
    }
    Ok(count)
}

fn blocks(r: &mut Reader<'_>, budget: &mut Budget) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let len = r.byte()? as usize;
        if len == 0 {
            return Ok(out);
        }
        budget.append(&mut out, r.take(len)?)?;
    }
}

fn skip_blocks(r: &mut Reader<'_>) -> Result<()> {
    loop {
        let len = r.byte()? as usize;
        if len == 0 {
            return Ok(());
        }
        r.take(len)?;
    }
}

fn lzw(data: &[u8], min: u8, len: usize, budget: &mut Budget) -> Result<Vec<u8>> {
    // Codes are packed least-significant bit first and grow in width with the dictionary.
    if !(2..=8).contains(&min) {
        return Err(DecodeError::InvalidData);
    }
    let clear = 1usize << min;
    let end = clear + 1;
    let mut next = clear + 2;
    let mut size = min + 1;
    let mut prefix = [0u16; 4096];
    let mut suffix = [0u8; 4096];
    let mut stack = [0u8; 4096];
    let mut previous = None;
    let mut first = 0;
    let mut bit = 0usize;
    let bit_len = data
        .len()
        .checked_mul(8)
        .ok_or(DecodeError::ImageTooLarge)?;
    // A code can expand to at most one byte per preceding code, capped by the
    // dictionary size. Reject impossible dimensions before allocating their output.
    let code_count = bit_len / usize::from(min + 1);
    let max_output = code_count.saturating_mul(code_count.min(prefix.len()));
    if len > max_output {
        return Err(DecodeError::InvalidData);
    }
    let mut output = budget.zeroed(len)?;
    let mut written = 0;
    loop {
        if bit + size as usize > bit_len {
            return Err(DecodeError::InvalidData);
        }
        let mut word = u32::from(data[bit / 8]);
        if let Some(v) = data.get(bit / 8 + 1) {
            word |= u32::from(*v) << 8;
        }
        if let Some(v) = data.get(bit / 8 + 2) {
            word |= u32::from(*v) << 16;
        }
        let code = ((word >> (bit % 8)) & ((1 << size) - 1)) as usize;
        bit += size as usize;
        if code == clear {
            next = clear + 2;
            size = min + 1;
            previous = None;
            continue;
        }
        if code == end {
            return if written == len {
                Ok(output)
            } else {
                Err(DecodeError::InvalidData)
            };
        }
        let mut current = code;
        let mut count = 0;
        if code == next
            && let Some(old) = previous
        {
            stack[count] = first;
            count += 1;
            current = old;
        } else if code >= next {
            return Err(DecodeError::InvalidData);
        }
        while current >= clear {
            if current >= next || count >= stack.len() - 1 {
                return Err(DecodeError::InvalidData);
            }
            stack[count] = suffix[current];
            count += 1;
            current = prefix[current] as usize;
        }
        first = current as u8;
        stack[count] = first;
        count += 1;
        if count > len - written {
            return Err(DecodeError::InvalidData);
        }
        for v in stack[..count].iter().rev() {
            output[written] = *v;
            written += 1;
        }
        if let Some(old) = previous
            && next < 4096
        {
            prefix[next] = old as u16;
            suffix[next] = first;
            next += 1;
            if next == 1 << size && size < 12 {
                size += 1;
            }
        }
        previous = Some(code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disposal_method_values_are_validated() {
        assert!(matches!(
            DisposalMethod::try_from(3),
            Ok(DisposalMethod::Previous)
        ));
        assert!(DisposalMethod::try_from(4).is_err());
    }

    #[test]
    fn gif_lzw_full_dictionary() {
        let image = decode(include_bytes!("../tests/fixtures/large.gif")).expect("large GIF");
        assert_eq!((image.width(), image.height()), (97, 89));
        assert_eq!(
            image.pixels(),
            include_bytes!("../tests/fixtures/large.gif.rgba")
        );
    }
    fn gif_frame(out: &mut Vec<u8>, x: u16, index: u8, disposal: u8, transparent: bool) {
        out.extend_from_slice(&[
            0x21,
            0xf9,
            4,
            (disposal << 2) | u8::from(transparent),
            1,
            0,
            0,
            0,
            0x2c,
        ]);
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&[0, 0, 1, 0, 1, 0, 0, 2]);
        // One-pixel stream: clear (4), palette index, end (5), all 3-bit codes.
        let codes = 4u16 | (u16::from(index) << 3) | (5 << 6);
        out.extend_from_slice(&[2, codes as u8, (codes >> 8) as u8, 0]);
    }

    #[test]
    fn gif_transparency_restore_background_and_previous() {
        for disposal in [2, 3] {
            let mut data = b"GIF89a\x02\0\x01\0\x81\0\0".to_vec();
            data.extend_from_slice(&[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]);
            gif_frame(&mut data, 0, 1, 0, true);
            gif_frame(&mut data, 0, 2, disposal, true);
            gif_frame(&mut data, 1, 3, 0, true);
            data.push(0x3b);
            let image = decode(&data).expect("GIF disposal");
            assert_eq!(image.frames()[0].pixels(), &[255, 0, 0, 255, 0, 0, 0, 0]);
            assert_eq!(image.frames()[1].pixels(), &[0, 255, 0, 255, 0, 0, 0, 0]);
            let left = if disposal == 2 {
                [0, 0, 0, 0]
            } else {
                [255, 0, 0, 255]
            };
            assert_eq!(
                image.frames()[2].pixels(),
                [left, [0, 0, 255, 255]].concat()
            );
        }
    }
}
