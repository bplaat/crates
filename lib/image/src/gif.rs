/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::mem;
use std::time::Duration;

use super::{
    Area, Budget, ColorSpace, DecodeError, Format, Image, LoopCount, Reader, Result, pixel_len,
};

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

struct Decoder<'a> {
    reader: Reader<'a>,
    width: u32,
    height: u32,
    pixel_len: usize,
    global_palette: [[u8; 4]; 256],
    global_palette_len: usize,
    budget: Budget,
    compressed: Vec<u8>,
    canvas: Vec<u8>,
    frames: Vec<super::Frame>,
    has_loop_extension: bool,
    loop_count: LoopCount,
    control: GraphicControl,
}

pub(super) fn decode(data: &[u8]) -> Result<Image> {
    Decoder::new(data)?.decode()
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Result<Self> {
        // Extension blocks configure the next frame. Image descriptors decompress palette indexes,
        // composite a displayed frame, and then apply its disposal operation.
        let mut reader = Reader::new(&data[6..]);
        let width = u32::from(reader.le16()?);
        let height = u32::from(reader.le16()?);
        let pixel_len = pixel_len(width, height)?;
        let packed = reader.byte()?;
        reader.byte()?; // Background index is not representable on a transparent RGBA canvas.
        reader.byte()?; // Pixel aspect ratio does not change the pixel canvas.
        let mut global_palette = [[0; 4]; 256];
        let global_palette_len = Self::read_table(&mut reader, packed, &mut global_palette)?;
        Ok(Self {
            reader,
            width,
            height,
            pixel_len,
            global_palette,
            global_palette_len,
            budget: Budget::default(),
            compressed: Vec::new(),
            canvas: Vec::new(),
            frames: Vec::new(),
            has_loop_extension: false,
            loop_count: LoopCount::Finite(1),
            control: GraphicControl::default(),
        })
    }

    fn decode(mut self) -> Result<Image> {
        loop {
            match self.reader.byte()? {
                0x21 => self.read_extension()?,
                0x2c => self.read_frame()?,
                0x3b => {
                    if self.frames.is_empty() {
                        return Err(DecodeError::InvalidData);
                    }
                    return Ok(Image {
                        format: Format::Gif,
                        color_space: ColorSpace::Srgb,
                        is_animated: self.has_loop_extension || self.frames.len() > 1,
                        frames: self.frames,
                        loop_count: self.loop_count,
                    });
                }
                _ => return Err(DecodeError::InvalidData),
            }
        }
    }

    fn read_extension(&mut self) -> Result<()> {
        match self.reader.byte()? {
            0xf9 => {
                if self.reader.byte()? != 4 {
                    return Err(DecodeError::InvalidData);
                }
                let flags = self.reader.byte()?;
                if flags & 0xe0 != 0 {
                    return Err(DecodeError::InvalidData);
                }
                self.control.disposal = DisposalMethod::try_from((flags >> 2) & 7)?;
                self.control.delay = Duration::from_millis(u64::from(self.reader.le16()?) * 10);
                let index = self.reader.byte()?;
                self.control.transparent_index = if flags & 1 != 0 { Some(index) } else { None };
                if self.reader.byte()? != 0 {
                    return Err(DecodeError::InvalidData);
                }
            }
            0xff => {
                let size = self.reader.byte()? as usize;
                if size != 11 {
                    return Err(DecodeError::InvalidData);
                }
                let name = self.reader.take(size)?;
                if matches!(name, b"NETSCAPE2.0" | b"ANIMEXTS1.0") {
                    self.compressed.clear();
                    Self::read_blocks(&mut self.reader, &mut self.budget, &mut self.compressed)?;
                    let bytes = &self.compressed;
                    if bytes.len() != 3 || bytes[0] != 1 {
                        return Err(DecodeError::InvalidData);
                    }
                    let repeats = u16::from_le_bytes([bytes[1], bytes[2]]);
                    self.has_loop_extension = true;
                    self.loop_count = if repeats == 0 {
                        LoopCount::Infinite
                    } else {
                        LoopCount::Finite(u32::from(repeats) + 1)
                    };
                } else {
                    Self::skip_blocks(&mut self.reader)?;
                }
            }
            0x01 => return Err(DecodeError::UnsupportedFeature), // Plain-text rendering.
            _ => Self::skip_blocks(&mut self.reader)?,
        }
        Ok(())
    }

    fn read_frame(&mut self) -> Result<()> {
        let control = mem::take(&mut self.control);
        let x = self.reader.le16()? as usize;
        let y = self.reader.le16()? as usize;
        let w = self.reader.le16()? as usize;
        let h = self.reader.le16()? as usize;
        let area = Area {
            x,
            y,
            width: w,
            height: h,
        }
        .validate(self.width, self.height)?;
        let flags = self.reader.byte()?;
        if flags & 0x18 != 0 {
            return Err(DecodeError::InvalidData);
        }
        let mut local = [[0; 4]; 256];
        let local_len = Self::read_table(&mut self.reader, flags, &mut local)?;
        // Keep the full 256-entry table so validated indexes need no per-pixel bounds checks.
        let (palette, palette_len) = if local_len != 0 {
            (&local, local_len)
        } else {
            (&self.global_palette, self.global_palette_len)
        };
        if palette_len == 0
            || control
                .transparent_index
                .is_some_and(|t| t as usize >= palette_len)
        {
            return Err(DecodeError::InvalidData);
        }
        let bg = [0; 4];
        let min_code = self.reader.byte()?;
        self.compressed.clear();
        Self::read_blocks(&mut self.reader, &mut self.budget, &mut self.compressed)?;
        let indices = lzw(&self.compressed, min_code, w * h, &mut self.budget)?;
        if indices.iter().copied().max().unwrap_or(0) as usize >= palette_len {
            return Err(DecodeError::InvalidData);
        }
        if self.canvas.is_empty() {
            self.canvas = self.budget.zeroed(self.pixel_len)?;
        }
        // Once the trailer follows, the canvas can become the output frame without a copy.
        let last_frame = self.reader.data.get(self.reader.pos) == Some(&0x3b);
        let previous = if !last_frame && control.disposal == DisposalMethod::Previous {
            Some(area.snapshot(&self.canvas, self.width as usize, &mut self.budget)?)
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
                let dest = ((y + row) * self.width as usize + x) * 4;
                let dest = self.canvas[dest..dest + w * 4].as_chunks_mut::<4>().0;
                let indices = &indices[source_row * w..(source_row + 1) * w];
                if let Some(transparent) = control.transparent_index {
                    for (p, &index) in dest.iter_mut().zip(indices) {
                        if index != transparent {
                            *p = palette[index as usize];
                        }
                    }
                } else {
                    for (p, &index) in dest.iter_mut().zip(indices) {
                        *p = palette[index as usize];
                    }
                }
                source_row += 1;
            }
        }
        let pixels = if last_frame {
            mem::take(&mut self.canvas)
        } else {
            self.budget.copy(&self.canvas)?
        };
        self.budget.frame(
            &mut self.frames,
            self.width,
            self.height,
            pixels,
            control.delay,
        )?;
        if last_frame {
            return Ok(());
        }
        match control.disposal {
            DisposalMethod::Background => {
                area.clear(&mut self.canvas, self.width as usize, bg);
            }
            DisposalMethod::Previous => {
                area.restore(
                    &mut self.canvas,
                    self.width as usize,
                    &previous.expect("previous canvas was saved"),
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn read_table(
        reader: &mut Reader<'_>,
        flags: u8,
        palette: &mut [[u8; 4]; 256],
    ) -> Result<usize> {
        if flags & 0x80 == 0 {
            return Ok(0);
        }
        let count = 2usize << (flags & 7);
        for color in &mut palette[..count] {
            color[..3].copy_from_slice(reader.take(3)?);
            color[3] = 255;
        }
        Ok(count)
    }

    fn read_blocks(reader: &mut Reader<'_>, budget: &mut Budget, out: &mut Vec<u8>) -> Result<()> {
        loop {
            let len = reader.byte()? as usize;
            if len == 0 {
                return Ok(());
            }
            budget.append(out, reader.take(len)?)?;
        }
    }

    fn skip_blocks(reader: &mut Reader<'_>) -> Result<()> {
        loop {
            let len = reader.byte()? as usize;
            if len == 0 {
                return Ok(());
            }
            reader.take(len)?;
        }
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
    // Every dictionary entry is the previous string plus the next string's first byte, which
    // always sits contiguously in the output, so entries are stored as output ranges.
    let mut offsets = [0usize; 4096];
    let mut lengths = [0usize; 4096];
    let mut previous: Option<(usize, usize)> = None;
    let bit_len = data
        .len()
        .checked_mul(8)
        .ok_or(DecodeError::ImageTooLarge)?;
    // A code can expand to at most one byte per preceding code, capped by the
    // dictionary size. Reject impossible dimensions before allocating their output.
    let code_count = bit_len / usize::from(min + 1);
    let max_output = code_count.saturating_mul(code_count.min(offsets.len()));
    if len > max_output {
        return Err(DecodeError::InvalidData);
    }
    let mut output = budget.zeroed(len)?;
    let mut written = 0;
    let mut acc = 0u64;
    let mut acc_bits = 0u32;
    let mut pos = 0;
    loop {
        while acc_bits <= 56 && pos < data.len() {
            acc |= u64::from(data[pos]) << acc_bits;
            acc_bits += 8;
            pos += 1;
        }
        if acc_bits < u32::from(size) {
            return Err(DecodeError::InvalidData);
        }
        let code = (acc & ((1 << size) - 1)) as usize;
        acc >>= size;
        acc_bits -= u32::from(size);
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
        let start = written;
        if code < clear {
            if written == len {
                return Err(DecodeError::InvalidData);
            }
            output[written] = code as u8;
            written += 1;
        } else if code < next {
            let (offset, count) = (offsets[code], lengths[code]);
            if count > len - written {
                return Err(DecodeError::InvalidData);
            }
            if count <= 16 && written + 16 <= len {
                // Copy short strings as one block; bytes past `count` are overwritten later.
                let block: [u8; 16] = output[offset..offset + 16].try_into().expect("16 bytes");
                output[written..written + 16].copy_from_slice(&block);
            } else {
                output.copy_within(offset..offset + count, written);
            }
            written += count;
        } else if code == next
            && let Some((offset, count)) = previous
        {
            // The code being defined: the previous string followed by its own first byte.
            if count >= len - written {
                return Err(DecodeError::InvalidData);
            }
            output.copy_within(offset..offset + count, written);
            output[written + count] = output[offset];
            written += count + 1;
        } else {
            return Err(DecodeError::InvalidData);
        }
        if let Some((offset, count)) = previous
            && next < 4096
        {
            offsets[next] = offset;
            lengths[next] = count + 1;
            next += 1;
            if next == 1 << size && size < 12 {
                size += 1;
            }
        }
        previous = Some((start, written - start));
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

    // A reference GIF LZW encoder that resets the dictionary once it is full.
    fn lzw_encode(data: &[u8], min: u8) -> Vec<u8> {
        let clear = 1u32 << min;
        let (mut out, mut acc, mut acc_bits) = (Vec::new(), 0u64, 0u32);
        let mut emit = |code: u32, size: u32| {
            acc |= u64::from(code) << acc_bits;
            acc_bits += size;
            while acc_bits >= 8 {
                out.push(acc as u8);
                acc >>= 8;
                acc_bits -= 8;
            }
        };
        let mut dictionary = std::collections::HashMap::new();
        let mut next = clear + 2;
        let mut size = u32::from(min) + 1;
        emit(clear, size);
        let mut prefix = u32::from(data[0]);
        for &byte in &data[1..] {
            if let Some(&code) = dictionary.get(&(prefix, byte)) {
                prefix = code;
                continue;
            }
            emit(prefix, size);
            if next < 4096 {
                dictionary.insert((prefix, byte), next);
                next += 1;
                if next > 1 << size && size < 12 {
                    size += 1;
                }
            } else {
                emit(clear, size);
                dictionary.clear();
                next = clear + 2;
                size = u32::from(min) + 1;
            }
            prefix = u32::from(byte);
        }
        emit(prefix, size);
        emit(clear + 1, size);
        if acc_bits > 0 {
            out.push(acc as u8);
        }
        out
    }

    #[test]
    fn lzw_round_trips_growth_resets_and_long_strings() {
        let mut state = 7u32;
        let mut data = Vec::new();
        for _ in 0..4000 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            // Short noisy runs grow the code width; long runs build long dictionary strings.
            let run = if state.is_multiple_of(16) {
                300
            } else {
                state as usize % 8 + 1
            };
            data.extend(std::iter::repeat_n((state >> 8) as u8 & 3, run));
        }
        for min in [2, 8] {
            let encoded = lzw_encode(&data, min);
            assert_eq!(
                lzw(&encoded, min, data.len(), &mut Budget::default()).as_deref(),
                Ok(&data[..])
            );
            assert!(lzw(&encoded, min, data.len() - 1, &mut Budget::default()).is_err());
        }
    }

    #[test]
    fn image_rs_interlaced_fixture_decodes() {
        let image = decode(include_bytes!("../tests/images/gif/anim/interlaced.gif"))
            .expect("interlaced GIF");
        assert_eq!((image.width(), image.height()), (32, 32));
        assert_eq!(image.frames().len(), 1);
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
    fn gif_animation_metadata_distinguishes_single_frame_playback() {
        let mut single_frame = b"GIF89a\x01\0\x01\0\x80\0\0".to_vec();
        single_frame.extend_from_slice(&[255, 0, 0, 0, 0, 0]);
        gif_frame(&mut single_frame, 0, 0, 0, false);
        single_frame.push(0x3b);

        let image = decode(&single_frame).expect("single-frame GIF");
        assert!(!image.is_animated());
        assert_eq!(image.loop_count(), LoopCount::Finite(1));

        let mut final_previous = single_frame[..19].to_vec();
        gif_frame(&mut final_previous, 0, 0, 3, false);
        final_previous.push(0x3b);
        let image = decode(&final_previous).expect("final frame with previous disposal");
        assert_eq!(image.frames()[0].pixels(), &[255, 0, 0, 255]);

        for (repeats, expected) in [(0u16, LoopCount::Infinite), (2, LoopCount::Finite(3))] {
            let mut animated = single_frame[..19].to_vec();
            animated.extend_from_slice(b"\x21\xff\x0bNETSCAPE2.0\x03\x01");
            animated.extend_from_slice(&repeats.to_le_bytes());
            animated.push(0);
            animated.extend_from_slice(&single_frame[19..]);
            let image = decode(&animated).expect("looping single-frame GIF");
            assert!(image.is_animated());
            assert_eq!(image.loop_count(), expected);
        }
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
            assert!(image.is_animated());
            assert_eq!(image.loop_count(), LoopCount::Finite(1));
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

    #[test]
    fn partial_first_frame_leaves_uncovered_canvas_transparent() {
        let mut data = b"GIF89a\x02\0\x01\0\x80\x01\0".to_vec();
        data.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
        gif_frame(&mut data, 0, 0, 2, false);
        gif_frame(&mut data, 1, 1, 0, false);
        data.push(0x3b);
        let image = decode(&data).expect("partial GIF frame");
        assert_eq!(image.frames()[0].pixels(), &[255, 0, 0, 255, 0, 0, 0, 0]);
        assert_eq!(image.frames()[1].pixels(), &[0, 0, 0, 0, 0, 0, 255, 255]);
    }
}
