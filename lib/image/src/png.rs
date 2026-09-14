/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use super::{
    Area, Budget, ColorSpace, DecodeError, Format, Frame, Image, Reader, Result, pixel_len,
};

struct Header {
    width: u32,
    height: u32,
    depth: u8,
    color: ColorType,
    channels: usize,
    interlace: InterlaceMethod,
}

#[derive(Clone, Copy)]
struct Control {
    area: Area,
    delay: Duration,
    dispose: DisposeOp,
    blend: BlendOp,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ColorType {
    Grayscale,
    Truecolor,
    Indexed,
    GrayscaleAlpha,
    TruecolorAlpha,
}

impl ColorType {
    const fn channels(self) -> usize {
        match self {
            Self::Grayscale | Self::Indexed => 1,
            Self::GrayscaleAlpha => 2,
            Self::Truecolor => 3,
            Self::TruecolorAlpha => 4,
        }
    }

    const fn accepts_depth(self, depth: u8) -> bool {
        match self {
            Self::Grayscale => matches!(depth, 1 | 2 | 4 | 8 | 16),
            Self::Indexed => matches!(depth, 1 | 2 | 4 | 8),
            _ => matches!(depth, 8 | 16),
        }
    }
}

impl TryFrom<u8> for ColorType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Grayscale),
            2 => Ok(Self::Truecolor),
            3 => Ok(Self::Indexed),
            4 => Ok(Self::GrayscaleAlpha),
            6 => Ok(Self::TruecolorAlpha),
            _ => Err(DecodeError::InvalidHeader),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InterlaceMethod {
    None,
    Adam7,
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Adam7),
            _ => Err(DecodeError::InvalidHeader),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DisposeOp {
    None,
    Background,
    Previous,
}

impl TryFrom<u8> for DisposeOp {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Background),
            2 => Ok(Self::Previous),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BlendOp {
    Source,
    Over,
}

impl TryFrom<u8> for BlendOp {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Source),
            1 => Ok(Self::Over),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum DataPhase {
    #[default]
    Before,
    In,
    After,
}

impl DataPhase {
    const fn has_idat(self) -> bool {
        !matches!(self, Self::Before)
    }

    fn observe(&mut self, kind: &[u8]) {
        if *self == Self::In && kind != b"IDAT" {
            *self = Self::After;
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum FrameData {
    #[default]
    None,
    Idat,
    Fdat {
        seen: bool,
    },
}

struct PngState {
    palette: [[u8; 4]; 256],
    palette_len: usize,
    transparency: Option<[u16; 3]>,
    seen_transparency: bool,
    animation: Option<(u32, u32)>,
    sequence: u32,
    control: Option<Control>,
    compressed: Vec<u8>,
    canvas: Vec<u8>,
    frames: Vec<Frame>,
    data_phase: DataPhase,
    frame_data: FrameData,
}

impl PngState {
    const fn new() -> Self {
        Self {
            palette: [[0, 0, 0, 255]; 256],
            palette_len: 0,
            transparency: None,
            seen_transparency: false,
            animation: None,
            sequence: 0,
            control: None,
            compressed: Vec::new(),
            canvas: Vec::new(),
            frames: Vec::new(),
            data_phase: DataPhase::Before,
            frame_data: FrameData::None,
        }
    }

    fn check_sequence(&mut self, value: u32) -> Result<()> {
        if value != self.sequence {
            return Err(DecodeError::InvalidData);
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or(DecodeError::InvalidData)?;
        Ok(())
    }

    fn finish_frame(&mut self, header: &Header, budget: &mut Budget) -> Result<()> {
        // Validate an excluded default image even though it is not emitted as a frame.
        let Some(control) = self.control else {
            header.raster(
                header.width,
                header.height,
                &self.compressed,
                &self.palette[..self.palette_len],
                self.transparency,
                budget,
            )?;
            return Ok(());
        };
        let pixels = header.raster(
            control.area.width as u32,
            control.area.height as u32,
            &self.compressed,
            &self.palette[..self.palette_len],
            self.transparency,
            budget,
        )?;
        let previous = if control.dispose == DisposeOp::Previous {
            Some(
                control
                    .area
                    .snapshot(&self.canvas, header.width as usize, budget)?,
            )
        } else {
            None
        };
        for (row, source) in pixels.chunks_exact(control.area.width * 4).enumerate() {
            let start = ((control.area.y + row) * header.width as usize + control.area.x) * 4;
            let dest = &mut self.canvas[start..start + source.len()];
            if control.blend == BlendOp::Source {
                dest.copy_from_slice(source);
            } else {
                for (dest, source) in dest
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(source.as_chunks::<4>().0.iter())
                {
                    over(dest, source);
                }
            }
        }
        let pixels = budget.copy(&self.canvas)?;
        budget.frame(&mut self.frames, pixels, control.delay)?;
        match control.dispose {
            DisposeOp::Background => {
                control
                    .area
                    .clear(&mut self.canvas, header.width as usize, [0; 4]);
            }
            DisposeOp::Previous => {
                control.area.restore(
                    &mut self.canvas,
                    header.width as usize,
                    &previous.expect("previous canvas was saved"),
                );
            }
            DisposeOp::None => {}
        }
        Ok(())
    }
}

pub(super) fn decode(data: &[u8]) -> Result<Image> {
    // Parse the chunk stream while retaining the state needed to turn each APNG frame data
    // stream into a fully composited canvas frame.
    let mut r = Reader::new(&data[8..]);
    let (kind, bytes) = chunk(&mut r)?;
    if kind != b"IHDR" || bytes.len() != 13 {
        return Err(DecodeError::InvalidHeader);
    }
    let mut h = Reader::new(bytes);
    let width = h.be32()?;
    let height = h.be32()?;
    let depth = h.byte()?;
    let color = ColorType::try_from(h.byte()?)?;
    if !color.accepts_depth(depth) || h.byte()? != 0 || h.byte()? != 0 {
        return Err(DecodeError::InvalidHeader);
    }
    let interlace = InterlaceMethod::try_from(h.byte()?)?;
    let len = pixel_len(width, height)?;
    let header = Header {
        width,
        height,
        depth,
        color,
        channels: color.channels(),
        interlace,
    };
    let mut budget = Budget::default();
    let mut state = PngState::new();
    loop {
        let (kind, bytes) = chunk(&mut r)?;
        state.data_phase.observe(kind);
        match kind {
            b"PLTE" => {
                if state.data_phase.has_idat()
                    || state.palette_len != 0
                    || state.seen_transparency
                    || matches!(color, ColorType::Grayscale | ColorType::GrayscaleAlpha)
                    || bytes.is_empty()
                    || bytes.len() % 3 != 0
                    || bytes.len() > 768
                {
                    return Err(DecodeError::InvalidData);
                }
                state.palette_len = bytes.len() / 3;
                if color == ColorType::Indexed && state.palette_len > 1 << depth {
                    return Err(DecodeError::InvalidData);
                }
                for (entry, rgb) in state
                    .palette
                    .iter_mut()
                    .zip(bytes.as_chunks::<3>().0.iter())
                {
                    entry[..3].copy_from_slice(rgb);
                }
            }
            b"tRNS" => {
                if state.data_phase.has_idat() || state.seen_transparency {
                    return Err(DecodeError::InvalidData);
                }
                state.seen_transparency = true;
                let mut t = Reader::new(bytes);
                match color {
                    ColorType::Grayscale if bytes.len() == 2 => {
                        let value = t.be16()?;
                        if depth < 16 && value >= 1 << depth {
                            return Err(DecodeError::InvalidData);
                        }
                        state.transparency = Some([value, value, value]);
                    }
                    ColorType::Truecolor if bytes.len() == 6 => {
                        let values = [t.be16()?, t.be16()?, t.be16()?];
                        if depth < 16 && values.iter().any(|v| *v > 255) {
                            return Err(DecodeError::InvalidData);
                        }
                        state.transparency = Some(values);
                    }
                    ColorType::Indexed
                        if state.palette_len != 0
                            && !bytes.is_empty()
                            && bytes.len() <= state.palette_len =>
                    {
                        for (entry, alpha) in state.palette.iter_mut().zip(bytes) {
                            entry[3] = *alpha;
                        }
                    }
                    _ => return Err(DecodeError::InvalidData),
                }
            }
            b"acTL" => {
                if state.data_phase.has_idat() || state.animation.is_some() || bytes.len() != 8 {
                    return Err(DecodeError::InvalidData);
                }
                let mut a = Reader::new(bytes);
                let count = a.be32()?;
                let plays = a.be32()?;
                if count == 0 {
                    return Err(DecodeError::InvalidData);
                }
                state.animation = Some((count, plays));
                state.canvas = budget.zeroed(len)?;
            }
            b"fcTL" => {
                if state.animation.is_none() || bytes.len() != 26 {
                    return Err(DecodeError::InvalidData);
                }
                if state.data_phase.has_idat() {
                    state.finish_frame(&header, &mut budget)?;
                    state.compressed.clear();
                } else if state.control.is_some() {
                    return Err(DecodeError::InvalidData);
                }
                let mut c = Reader::new(bytes);
                state.check_sequence(c.be32()?)?;
                let w = c.be32()? as usize;
                let h = c.be32()? as usize;
                let x = c.be32()? as usize;
                let y = c.be32()? as usize;
                let area = Area {
                    x,
                    y,
                    width: w,
                    height: h,
                }
                .validate(width, height)?;
                if !state.data_phase.has_idat()
                    && (x != 0 || y != 0 || w != width as usize || h != height as usize)
                {
                    return Err(DecodeError::InvalidData);
                }
                let num = c.be16()?;
                let den = c.be16()?;
                let delay = Duration::from_nanos(
                    u64::from(num) * 1_000_000_000 / u64::from(if den == 0 { 100 } else { den }),
                );
                let dispose = DisposeOp::try_from(c.byte()?)?;
                let blend = BlendOp::try_from(c.byte()?)?;
                state.frame_data = if state.data_phase.has_idat() {
                    FrameData::Fdat { seen: false }
                } else {
                    FrameData::Idat
                };
                state.control = Some(Control {
                    area,
                    delay,
                    dispose,
                    blend,
                });
            }
            b"IDAT" => {
                if state.data_phase == DataPhase::After
                    || (color == ColorType::Indexed && state.palette_len == 0)
                {
                    return Err(DecodeError::InvalidData);
                }
                state.data_phase = DataPhase::In;
                budget.append(&mut state.compressed, bytes)?;
            }
            b"fdAT" => {
                if !state.data_phase.has_idat()
                    || !matches!(state.frame_data, FrameData::Fdat { .. })
                    || state.control.is_none()
                    || state.animation.is_none()
                    || bytes.len() < 4
                {
                    return Err(DecodeError::InvalidData);
                }
                let mut f = Reader::new(bytes);
                state.check_sequence(f.be32()?)?;
                budget.append(&mut state.compressed, f.take(f.remaining())?)?;
                state.frame_data = FrameData::Fdat { seen: true };
            }
            b"IEND" => {
                if !bytes.is_empty() || !state.data_phase.has_idat() || r.remaining() != 0 {
                    return Err(DecodeError::InvalidData);
                }
                if state.animation.is_none() {
                    let pixels = header.raster(
                        width,
                        height,
                        &state.compressed,
                        &state.palette[..state.palette_len],
                        state.transparency,
                        &mut budget,
                    )?;
                    return Ok(Image::still(Format::Png, width, height, pixels));
                }
                // A non-default animation frame must have an fdAT stream.
                if state.frame_data == (FrameData::Fdat { seen: false }) {
                    return Err(DecodeError::InvalidData);
                }
                state.finish_frame(&header, &mut budget)?;
                let (count, loop_count) = state.animation.expect("animation was checked");
                if state.frames.len() != count as usize {
                    return Err(DecodeError::InvalidData);
                }
                return Ok(Image {
                    format: Format::Png,
                    width,
                    height,
                    color_space: ColorSpace::Srgb,
                    frames: state.frames,
                    loop_count,
                });
            }
            _ if kind[0] & 32 == 0 => return Err(DecodeError::UnsupportedFeature),
            _ => {}
        }
    }
}

fn chunk<'a>(r: &mut Reader<'a>) -> Result<(&'a [u8], &'a [u8])> {
    let len = r.be32()? as usize;
    if len > 0x7fff_ffff {
        return Err(DecodeError::InvalidData);
    }
    let kind = r.take(4)?;
    if !kind.iter().all(u8::is_ascii_alphabetic) || kind[2] & 32 != 0 {
        return Err(DecodeError::InvalidData);
    }
    let bytes = r.take(len)?;
    let expected = r.be32()?;
    let mut crc = !0u32;
    for byte in kind.iter().chain(bytes) {
        crc = CRC_TABLE[((crc as u8) ^ byte) as usize] ^ (crc >> 8);
    }
    if !crc != expected {
        return Err(DecodeError::InvalidData);
    }
    Ok((kind, bytes))
}

const CRC_TABLE: [u32; 256] = {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut j = 0;
        while j < 8 {
            c = (c >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(c & 1));
            j += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};

fn over(d: &mut [u8], s: &[u8]) {
    let sa = u32::from(s[3]);
    let da = u32::from(d[3]) * (255 - sa);
    let alpha = sa * 255 + da;
    if alpha == 0 {
        d.fill(0);
        return;
    }
    for channel in 0..3 {
        d[channel] = ((u32::from(s[channel]) * sa * 255 + u32::from(d[channel]) * da + alpha / 2)
            / alpha) as u8;
    }
    d[3] = ((alpha + 127) / 255) as u8;
}

const ADAM7: [(usize, usize, usize, usize); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

impl Header {
    fn raster(
        &self,
        width: u32,
        height: u32,
        compressed: &[u8],
        palette: &[[u8; 4]],
        transparency: Option<[u16; 3]>,
        budget: &mut Budget,
    ) -> Result<Vec<u8>> {
        // Adam7 writes seven sparse passes into one output; ordinary PNG uses one full-size pass.
        let passes = if self.interlace == InterlaceMethod::Adam7 {
            &ADAM7[..]
        } else {
            &[(0, 0, 1, 1)][..]
        };
        let width = width as usize;
        let height = height as usize;
        let mut expected = 0usize;
        for &(x, y, dx, dy) in passes {
            let w = width.saturating_sub(x).div_ceil(dx);
            let rows = height.saturating_sub(y).div_ceil(dy);
            if w > 0 && rows > 0 {
                expected = expected
                    .checked_add(
                        png_row_bytes(w, self.channels, self.depth)?
                            .checked_add(1)
                            .ok_or(DecodeError::ImageTooLarge)?
                            .checked_mul(rows)
                            .ok_or(DecodeError::ImageTooLarge)?,
                    )
                    .ok_or(DecodeError::ImageTooLarge)?;
            }
        }
        budget.claim(expected)?;
        // Decompress into an exactly sized, fallibly allocated buffer; no unbounded growth.
        let mut raw = Vec::new();
        raw.try_reserve_exact(expected)
            .map_err(|_| DecodeError::ImageTooLarge)?;
        raw.resize(expected, 0);
        let mut inflater = miniz_oxide::inflate::core::DecompressorOxide::new();
        let flags = miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
            | miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
        let (status, consumed, written) =
            miniz_oxide::inflate::core::decompress(&mut inflater, compressed, &mut raw, 0, flags);
        if status != miniz_oxide::inflate::TINFLStatus::Done
            || consumed != compressed.len()
            || written != expected
        {
            return Err(DecodeError::InvalidData);
        }
        let mut pixels = budget.zeroed(pixel_len(width as u32, height as u32)?)?;
        let bpp = (self.channels * self.depth as usize).div_ceil(8);
        let mut cursor = 0;
        for &(x, y, dx, dy) in passes {
            let w = width.saturating_sub(x).div_ceil(dx);
            let rows = height.saturating_sub(y).div_ceil(dy);
            if w == 0 || rows == 0 {
                continue;
            }
            let bytes = png_row_bytes(w, self.channels, self.depth)?;
            let mut previous = None;
            for row in 0..rows {
                let filter = raw[cursor];
                cursor += 1;
                let scan_start = cursor;
                let scan = if let Some(previous_start) = previous {
                    let (before, current) = raw.split_at_mut(scan_start);
                    let previous = &before[previous_start..previous_start + bytes];
                    let scan = &mut current[..bytes];
                    unfilter(scan, Some(previous), bpp, filter)?;
                    scan
                } else {
                    let scan = &mut raw[scan_start..scan_start + bytes];
                    unfilter(scan, None, bpp, filter)?;
                    scan
                };
                for col in 0..w {
                    let dest = ((y + row * dy) * width + x + col * dx) * 4;
                    let p = &mut pixels[dest..dest + 4];
                    match self.color {
                        ColorType::Grayscale if self.depth == 16 => {
                            let sample = sample16(scan, col * 2)?;
                            let value = scale_16_to_8(sample);
                            p.copy_from_slice(&[
                                value,
                                value,
                                value,
                                if transparency.is_some_and(|t| t[0] == sample) {
                                    0
                                } else {
                                    255
                                },
                            ]);
                        }
                        ColorType::Grayscale | ColorType::Indexed => {
                            let bit = col * self.depth as usize;
                            let sample = (scan[bit / 8] >> (8 - self.depth as usize - bit % 8))
                                & ((1u16 << self.depth) - 1) as u8;
                            if self.color == ColorType::Indexed {
                                p.copy_from_slice(
                                    palette
                                        .get(sample as usize)
                                        .ok_or(DecodeError::InvalidData)?,
                                );
                            } else {
                                let value =
                                    (u16::from(sample) * 255 / ((1 << self.depth) - 1)) as u8;
                                p.copy_from_slice(&[
                                    value,
                                    value,
                                    value,
                                    if transparency.is_some_and(|t| t[0] == u16::from(sample)) {
                                        0
                                    } else {
                                        255
                                    },
                                ]);
                            }
                        }
                        ColorType::Truecolor => {
                            if self.depth == 16 {
                                let offset = col * 6;
                                let samples = [
                                    sample16(scan, offset)?,
                                    sample16(scan, offset + 2)?,
                                    sample16(scan, offset + 4)?,
                                ];
                                p.copy_from_slice(&[
                                    scale_16_to_8(samples[0]),
                                    scale_16_to_8(samples[1]),
                                    scale_16_to_8(samples[2]),
                                    if transparency == Some(samples) {
                                        0
                                    } else {
                                        255
                                    },
                                ]);
                            } else {
                                let s = &scan[col * 3..col * 3 + 3];
                                p[..3].copy_from_slice(s);
                                p[3] = if transparency
                                    == Some([u16::from(s[0]), u16::from(s[1]), u16::from(s[2])])
                                {
                                    0
                                } else {
                                    255
                                };
                            }
                        }
                        ColorType::GrayscaleAlpha => {
                            if self.depth == 16 {
                                let offset = col * 4;
                                let value = scale_16_to_8(sample16(scan, offset)?);
                                let alpha = scale_16_to_8(sample16(scan, offset + 2)?);
                                p.copy_from_slice(&[value, value, value, alpha]);
                            } else {
                                let s = &scan[col * 2..col * 2 + 2];
                                p.copy_from_slice(&[s[0], s[0], s[0], s[1]]);
                            }
                        }
                        ColorType::TruecolorAlpha => {
                            if self.depth == 16 {
                                let offset = col * 8;
                                for (channel, value) in p.iter_mut().enumerate() {
                                    *value = scale_16_to_8(sample16(scan, offset + channel * 2)?);
                                }
                            } else {
                                p.copy_from_slice(&scan[col * 4..col * 4 + 4]);
                            }
                        }
                    }
                }
                previous = Some(scan_start);
                cursor += bytes;
            }
        }
        Ok(pixels)
    }
}

fn png_row_bytes(width: usize, channels: usize, depth: u8) -> Result<usize> {
    width
        .checked_mul(channels)
        .and_then(|samples| samples.checked_mul(depth as usize))
        .and_then(|bits| bits.checked_add(7))
        .map(|bits| bits / 8)
        .ok_or(DecodeError::ImageTooLarge)
}

fn sample16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_be_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or(DecodeError::InvalidData)?
            .try_into()
            .expect("two bytes"),
    ))
}

const fn scale_16_to_8(value: u16) -> u8 {
    ((value as u32 + 128) / 257) as u8
}

fn unfilter(row: &mut [u8], previous: Option<&[u8]>, bpp: usize, filter: u8) -> Result<()> {
    if previous.is_some_and(|previous| row.len() != previous.len()) {
        return Err(DecodeError::InvalidData);
    }
    match (filter, previous) {
        (0, _) | (2, None) => {}
        (1 | 4, None) | (1, Some(_)) => {
            for i in bpp..row.len() {
                row[i] = row[i].wrapping_add(row[i - bpp]);
            }
        }
        (2, Some(previous)) => {
            add_bytes(row, previous);
        }
        (3, None) => {
            for i in bpp..row.len() {
                row[i] = row[i].wrapping_add(row[i - bpp] / 2);
            }
        }
        (filter @ (3 | 4), Some(previous)) => {
            for i in 0..row.len() {
                let a = if i >= bpp { row[i - bpp] } else { 0 };
                let b = previous[i];
                let c = if i >= bpp { previous[i - bpp] } else { 0 };
                let predictor = if filter == 3 {
                    ((u16::from(a) + u16::from(b)) / 2) as u8
                } else {
                    paeth(a, b, c)
                };
                row[i] = row[i].wrapping_add(predictor);
            }
        }
        _ => return Err(DecodeError::InvalidData),
    }
    Ok(())
}

fn add_bytes(dest: &mut [u8], source: &[u8]) {
    let (dest_chunks, dest_tail) = dest.as_chunks_mut::<16>();
    let (source_chunks, source_tail) = source.as_chunks::<16>();
    for (dest, source) in dest_chunks.iter_mut().zip(source_chunks) {
        for (dest, source) in dest.iter_mut().zip(source) {
            *dest = dest.wrapping_add(*source);
        }
    }
    for (dest, source) in dest_tail.iter_mut().zip(source_tail) {
        *dest = dest.wrapping_add(*source);
    }
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let pa = (p - i16::from(a)).abs();
    let pb = (p - i16::from(b)).abs();
    let pc = (p - i16::from(c)).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_BYTES;

    #[test]
    fn format_values_are_validated() {
        assert!(matches!(
            ColorType::try_from(6),
            Ok(ColorType::TruecolorAlpha)
        ));
        assert!(ColorType::try_from(1).is_err());
        assert!(matches!(
            InterlaceMethod::try_from(1),
            Ok(InterlaceMethod::Adam7)
        ));
        assert!(InterlaceMethod::try_from(2).is_err());
        assert!(DisposeOp::try_from(3).is_err());
        assert!(BlendOp::try_from(2).is_err());
    }

    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        let mut crc = !0u32;
        for byte in kind.iter().chain(data) {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        out.extend_from_slice(&(!crc).to_be_bytes());
    }

    fn png_header(width: u32, height: u32, depth: u8, color: u8) -> Vec<u8> {
        png_header_with_interlace(width, height, depth, color, 0)
    }

    fn png_header_with_interlace(
        width: u32,
        height: u32,
        depth: u8,
        color: u8,
        interlace: u8,
    ) -> Vec<u8> {
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = width.to_be_bytes().to_vec();
        header.extend_from_slice(&height.to_be_bytes());
        header.extend_from_slice(&[depth, color, 0, 0, interlace]);
        chunk(&mut out, b"IHDR", &header);
        out
    }

    fn png_finish(mut out: Vec<u8>, scanlines: &[u8]) -> Vec<u8> {
        chunk(
            &mut out,
            b"IDAT",
            &miniz_oxide::deflate::compress_to_vec_zlib(scanlines, 6),
        );
        chunk(&mut out, b"IEND", &[]);
        out
    }

    #[test]
    fn decodes_16_bit_png_types_to_rgba8() {
        let mut grayscale = png_header(2, 1, 16, 0);
        chunk(&mut grayscale, b"tRNS", &[0xab, 0xcd]);
        assert_eq!(
            decode(&png_finish(grayscale, &[0, 0x12, 0x34, 0xab, 0xcd]))
                .expect("16-bit grayscale")
                .pixels(),
            &[0x12, 0x12, 0x12, 255, 0xab, 0xab, 0xab, 0]
        );

        let mut truecolor = png_header(1, 1, 16, 2);
        chunk(
            &mut truecolor,
            b"tRNS",
            &[0x12, 0x35, 0x56, 0x78, 0x9a, 0xbc],
        );
        assert_eq!(
            decode(&png_finish(
                truecolor,
                &[0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]
            ))
            .expect("16-bit truecolor")
            .pixels(),
            &[0x12, 0x56, 0x9a, 255]
        );

        assert_eq!(
            decode(&png_finish(
                png_header(1, 1, 16, 4),
                &[0, 0x12, 0x34, 0xab, 0xcd]
            ))
            .expect("16-bit grayscale-alpha")
            .pixels(),
            &[0x12, 0x12, 0x12, 0xab]
        );
        assert_eq!(
            decode(&png_finish(
                png_header_with_interlace(1, 1, 16, 6, 1),
                &[0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]
            ))
            .expect("16-bit Adam7 RGBA")
            .pixels(),
            &[0x12, 0x56, 0x9a, 0xde]
        );
    }

    #[test]
    fn packed_grayscale_palette_and_transparency() {
        for depth in [1, 2, 4, 8] {
            let max = ((1u16 << depth) - 1) as u8;
            let mut h = png_header(2, 1, depth, 0);
            chunk(&mut h, b"tRNS", &[0, max]);
            let scan = if depth == 8 {
                vec![0, 0, max]
            } else {
                vec![0, max << (8 - 2 * depth)]
            };
            assert_eq!(
                decode(&png_finish(h, &scan)).expect("gray").pixels(),
                &[0, 0, 0, 255, 255, 255, 255, 0]
            );
            let mut h = png_header(2, 1, depth, 3);
            chunk(&mut h, b"PLTE", &[255, 0, 0, 0, 255, 0]);
            chunk(&mut h, b"tRNS", &[255, 50]);
            let scan = if depth == 8 {
                vec![0, 0, 1]
            } else {
                vec![0, 1 << (8 - 2 * depth)]
            };
            assert_eq!(
                decode(&png_finish(h, &scan)).expect("palette").pixels(),
                &[255, 0, 0, 255, 0, 255, 0, 50]
            );
        }
        let mut h = png_header(1, 1, 8, 2);
        chunk(&mut h, b"tRNS", &[0, 10, 0, 20, 0, 30]);
        assert_eq!(
            decode(&png_finish(h, &[0, 10, 20, 30]))
                .expect("RGB transparency")
                .pixels(),
            &[10, 20, 30, 0]
        );
    }

    fn frame_control(out: &mut Vec<u8>, seq: u32, area: [u32; 4], disposal: u8, blend: u8) {
        let mut bytes = seq.to_be_bytes().to_vec();
        for v in area {
            bytes.extend_from_slice(&v.to_be_bytes());
        }
        bytes.extend_from_slice(&[0, 1, 0, 10, disposal, blend]);
        chunk(out, b"fcTL", &bytes);
    }

    #[test]
    fn apng_blending_disposal_and_excluded_default() {
        for excluded in [false, true] {
            let mut out = png_header(2, 1, 8, 6);
            chunk(&mut out, b"acTL", &[0, 0, 0, 4, 0, 0, 0, 2]);
            let compressed =
                miniz_oxide::deflate::compress_to_vec_zlib(&[0, 255, 0, 0, 255, 255, 0, 0, 255], 6);
            if excluded {
                chunk(&mut out, b"IDAT", &compressed);
            }
            frame_control(&mut out, 0, [2, 1, 0, 0], 0, 0);
            let mut seq = 1u32;
            if excluded {
                let mut bytes = seq.to_be_bytes().to_vec();
                bytes.extend_from_slice(&compressed);
                chunk(&mut out, b"fdAT", &bytes);
                seq += 1;
            } else {
                chunk(&mut out, b"IDAT", &compressed);
            }
            frame_control(&mut out, seq, [1, 1, 0, 0], 2, 1);
            seq += 1;
            let mut bytes = seq.to_be_bytes().to_vec();
            bytes.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
                &[0, 0, 0, 255, 128],
                6,
            ));
            chunk(&mut out, b"fdAT", &bytes);
            seq += 1;
            frame_control(&mut out, seq, [1, 1, 1, 0], 1, 0);
            seq += 1;
            let mut bytes = seq.to_be_bytes().to_vec();
            bytes.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
                &[0, 0, 255, 0, 255],
                6,
            ));
            chunk(&mut out, b"fdAT", &bytes);
            seq += 1;
            frame_control(&mut out, seq, [1, 1, 0, 0], 0, 1);
            seq += 1;
            let mut bytes = seq.to_be_bytes().to_vec();
            bytes.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(&[0; 5], 6));
            chunk(&mut out, b"fdAT", &bytes);
            chunk(&mut out, b"IEND", &[]);
            let image = decode(&out).expect("APNG");
            assert_eq!(image.loop_count(), 2);
            assert_eq!(image.frames().len(), 4);
            assert_eq!(image.frames()[3].pixels(), &[255, 0, 0, 255, 0, 0, 0, 0]);
            assert_eq!(
                image.frames()[1].pixels(),
                &[127, 0, 128, 255, 255, 0, 0, 255]
            );
            assert_eq!(
                image.frames()[2].pixels(),
                &[255, 0, 0, 255, 0, 255, 0, 255]
            );
            assert!(
                image
                    .frames()
                    .iter()
                    .all(|f| f.delay() == Duration::from_millis(100))
            );
        }
    }

    #[test]
    fn allocation_limits_and_bad_png_streams() {
        assert_eq!(
            pixel_len(u32::MAX, u32::MAX),
            Err(DecodeError::ImageTooLarge)
        );
        assert_eq!(
            decode(&png_header(100_000, 100_000, 8, 6)),
            Err(DecodeError::ImageTooLarge)
        );
        assert!(decode(&png_finish(png_header(1, 1, 8, 6), &[0; 10000])).is_err());
        assert!(decode(&png_finish(png_header(1, 1, 8, 6), &[9, 0, 0, 0, 0])).is_err());
        let mut budget = Budget::default();
        budget.claim(MAX_BYTES).expect("limit");
        assert_eq!(budget.claim(1), Err(DecodeError::ImageTooLarge));
    }

    #[test]
    fn png_all_filters_reconstruct_rows() {
        let rows = [
            [10u8, 20, 30, 255, 40, 50, 60, 128],
            [15, 10, 50, 200, 20, 70, 90, 100],
        ];
        for filter in 0..=4 {
            let mut scan = Vec::new();
            for (y, row) in rows.iter().enumerate() {
                scan.push(filter);
                for i in 0..row.len() {
                    let a = if i >= 4 { row[i - 4] } else { 0 };
                    let b = if y > 0 { rows[y - 1][i] } else { 0 };
                    let c = if y > 0 && i >= 4 {
                        rows[y - 1][i - 4]
                    } else {
                        0
                    };
                    let predict = match filter {
                        0 => 0,
                        1 => a,
                        2 => b,
                        3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                        4 => {
                            let p = i32::from(a) + i32::from(b) - i32::from(c);
                            let (_, v) = [
                                (i32::from(a) - p).abs(),
                                (i32::from(b) - p).abs(),
                                (i32::from(c) - p).abs(),
                            ]
                            .into_iter()
                            .zip([a, b, c])
                            .min_by_key(|v| v.0)
                            .expect("three predictors");
                            v
                        }
                        _ => unreachable!(),
                    };
                    scan.push(row[i].wrapping_sub(predict));
                }
            }
            assert_eq!(
                decode(&png_finish(png_header(2, 2, 8, 6), &scan))
                    .expect("filtered PNG")
                    .pixels(),
                rows.concat()
            );
        }
    }

    #[test]
    fn png_16_bit_filters_reconstruct_rows() {
        let rows = [
            [
                0x12u8, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x21, 0x43, 0x65, 0x87, 0xa9,
                0xcb, 0xed, 0x0f,
            ],
            [
                0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
                0xfe, 0x10,
            ],
        ];
        for filter in 0..=4 {
            let mut scan = Vec::new();
            for (y, row) in rows.iter().enumerate() {
                scan.push(filter);
                for i in 0..row.len() {
                    let a = if i >= 8 { row[i - 8] } else { 0 };
                    let b = if y > 0 { rows[y - 1][i] } else { 0 };
                    let c = if y > 0 && i >= 8 {
                        rows[y - 1][i - 8]
                    } else {
                        0
                    };
                    let predictor = match filter {
                        0 => 0,
                        1 => a,
                        2 => b,
                        3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                        4 => paeth(a, b, c),
                        _ => unreachable!(),
                    };
                    scan.push(row[i].wrapping_sub(predictor));
                }
            }
            assert_eq!(
                decode(&png_finish(png_header(2, 2, 16, 6), &scan))
                    .expect("filtered 16-bit PNG")
                    .pixels(),
                &[
                    0x12, 0x56, 0x9a, 0xde, 0x21, 0x65, 0xa9, 0xec, 0x23, 0x67, 0xab, 0xee, 0x32,
                    0x76, 0xba, 0xfd,
                ]
            );
        }
    }

    #[test]
    fn scales_16_bit_samples_to_nearest_8_bit_value() {
        assert_eq!(scale_16_to_8(0), 0);
        assert_eq!(scale_16_to_8(129), 1);
        assert_eq!(scale_16_to_8(65_535), 255);
    }

    #[test]
    fn image_rs_16_bit_apng_and_grayscale_alpha() {
        let image =
            decode(include_bytes!("../tests/images/png/apng/rgba16.png")).expect("16-bit APNG");
        assert_eq!(image.frames().len(), 3);
        for (frame, expected) in
            image
                .frames()
                .iter()
                .zip([[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]])
        {
            assert!(
                frame
                    .pixels()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|pixel| *pixel == expected)
            );
        }
        assert_eq!(
            decode(&png_finish(png_header(2, 1, 8, 4), &[0, 20, 30, 40, 50]))
                .expect("gray alpha")
                .pixels(),
            &[20, 20, 20, 30, 40, 40, 40, 50]
        );
    }
}
