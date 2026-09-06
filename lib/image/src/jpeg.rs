/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{Budget, DecodeError, Format, Image, Reader, Result, pixel_len};

const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

struct Huffman {
    first: [u32; 17],
    count: [u32; 17],
    offset: [usize; 17],
    values: [u8; 256],
}

impl Huffman {
    fn read(r: &mut Reader<'_>) -> Result<Self> {
        // Store canonical-code ranges by bit length for direct symbol lookup while decoding.
        let counts = r.take(16)?;
        let mut first = [0; 17];
        let mut count = [0; 17];
        let mut offset = [0; 17];
        let mut code = 0;
        let mut total = 0;
        for len in 1..=16 {
            first[len] = code;
            count[len] = u32::from(counts[len - 1]);
            offset[len] = total;
            total += count[len] as usize;
            code += count[len];
            // JPEG reserves the all-ones code for entropy padding.
            if code >= 1 << len {
                return Err(DecodeError::InvalidData);
            }
            code <<= 1;
        }
        if total == 0 || total > 256 {
            return Err(DecodeError::InvalidData);
        }
        let mut values = [0; 256];
        values[..total].copy_from_slice(r.take(total)?);
        Ok(Self {
            first,
            count,
            offset,
            values,
        })
    }

    fn symbol(&self, bits: &mut Bits<'_, '_>) -> Result<u8> {
        let mut code = 0;
        for len in 1..=16 {
            code = (code << 1) | bits.get(1)?;
            let delta = code.wrapping_sub(self.first[len]);
            if delta < self.count[len] {
                return Ok(self.values[self.offset[len] + delta as usize]);
            }
        }
        Err(DecodeError::InvalidData)
    }
}

struct Bits<'a, 'b> {
    r: &'b mut Reader<'a>,
    byte: u8,
    left: u8,
}

impl Bits<'_, '_> {
    fn get(&mut self, count: u8) -> Result<u32> {
        // JPEG escapes literal 0xff entropy bytes with a zero byte.
        let mut result = 0;
        for _ in 0..count {
            if self.left == 0 {
                self.byte = self.r.byte()?;
                if self.byte == 255 && self.r.byte()? != 0 {
                    return Err(DecodeError::InvalidData);
                }
                self.left = 8;
            }
            self.left -= 1;
            result = (result << 1) | u32::from((self.byte >> self.left) & 1);
        }
        Ok(result)
    }

    fn signed(&mut self, size: u8) -> Result<i32> {
        if size == 0 {
            return Ok(0);
        }
        let value = self.get(size)? as i32;
        Ok(if value < 1 << (size - 1) {
            value + 1 - (1 << size)
        } else {
            value
        })
    }

    const fn align(&mut self) -> Result<()> {
        if self.left != 0 && self.byte & ((1 << self.left) - 1) != (1 << self.left) - 1 {
            return Err(DecodeError::InvalidData);
        }
        self.left = 0;
        Ok(())
    }
}

struct Component {
    id: u8,
    h: usize,
    v: usize,
    quant_id: usize,
    blocks_w: usize,
    blocks_h: usize,
    actual_w: usize,
    actual_h: usize,
    coefficients: Vec<i32>,
    quant: Option<[u16; 64]>,
    approximation: [i8; 64],
}

struct Jpeg {
    width: u32,
    height: u32,
    max_h: usize,
    max_v: usize,
    mcus_w: usize,
    mcus_h: usize,
    progressive: bool,
    components: Vec<Component>,
    quant: [Option<[u16; 64]>; 4],
    dc: [Option<Huffman>; 4],
    ac: [Option<Huffman>; 4],
    restart: usize,
    adobe: Option<u8>,
    jfif: bool,
    orientation: u16,
}

pub(super) fn decode(data: &[u8]) -> Result<Image> {
    let mut r = Reader::new(&data[2..]);
    let mut budget = Budget::default();
    let mut j = Jpeg {
        width: 0,
        height: 0,
        max_h: 0,
        max_v: 0,
        mcus_w: 0,
        mcus_h: 0,
        progressive: false,
        components: Vec::new(),
        quant: [None; 4],
        dc: [None, None, None, None],
        ac: [None, None, None, None],
        restart: 0,
        adobe: None,
        jfif: false,
        orientation: 1,
    };
    let mut scans = 0;
    loop {
        let m = marker(&mut r)?;
        if m == 0xd9 {
            if scans == 0 || j.components.iter().any(|c| c.approximation[0] < 0) {
                return Err(DecodeError::InvalidData);
            }
            return j.render(&mut budget);
        }
        if matches!(m, 0xd0..=0xd8 | 0x01) {
            return Err(DecodeError::InvalidData);
        }
        let len = r.be16()?;
        if len < 2 {
            return Err(DecodeError::InvalidData);
        }
        let bytes = r.take(len as usize - 2)?;
        let mut segment = Reader::new(bytes);
        match m {
            0xc0..=0xc2 => j.header(&mut segment, m == 0xc2, &mut budget)?,
            0xc4 => {
                while segment.remaining() > 0 {
                    let id = segment.byte()?;
                    if id & 15 > 3 || id >> 4 > 1 {
                        return Err(DecodeError::InvalidData);
                    }
                    budget.claim(size_of::<Huffman>() + 256)?;
                    let table = Huffman::read(&mut segment)?;
                    if id >> 4 == 0 {
                        j.dc[(id & 15) as usize] = Some(table);
                    } else {
                        j.ac[(id & 15) as usize] = Some(table);
                    }
                }
            }
            0xdb => {
                while segment.remaining() > 0 {
                    let id = segment.byte()?;
                    if id & 15 > 3 || id >> 4 > 1 {
                        return Err(DecodeError::InvalidData);
                    }
                    let mut q = [0; 64];
                    for k in ZIGZAG {
                        q[k] = if id >> 4 == 0 {
                            u16::from(segment.byte()?)
                        } else {
                            segment.be16()?
                        };
                        if q[k] == 0 {
                            return Err(DecodeError::InvalidData);
                        }
                    }
                    j.quant[(id & 15) as usize] = Some(q);
                }
            }
            0xdd => {
                j.restart = segment.be16()? as usize;
                if segment.remaining() != 0 {
                    return Err(DecodeError::InvalidData);
                }
            }
            0xda => {
                j.scan(&mut segment, &mut r)?;
                scans += 1;
            }
            0xe0 => {
                if bytes.starts_with(b"JFIF\0") {
                    j.jfif = true;
                }
            }
            0xe1 => {
                if let Some(orientation) = exif_orientation(bytes) {
                    j.orientation = orientation;
                }
            }
            0xee => {
                if bytes.starts_with(b"Adobe") && bytes.len() >= 12 {
                    j.adobe = Some(bytes[11]);
                }
            }
            0xe2..=0xed | 0xef | 0xfe => {}
            _ => return Err(DecodeError::UnsupportedFeature),
        }
    }
}

fn marker(r: &mut Reader<'_>) -> Result<u8> {
    if r.byte()? != 255 {
        return Err(DecodeError::InvalidData);
    }
    loop {
        let byte = r.byte()?;
        if byte == 0 {
            return Err(DecodeError::InvalidData);
        }
        if byte != 255 {
            return Ok(byte);
        }
    }
}

impl Jpeg {
    fn header(&mut self, r: &mut Reader<'_>, progressive: bool, budget: &mut Budget) -> Result<()> {
        if !self.components.is_empty() {
            return Err(DecodeError::InvalidData);
        }
        if r.byte()? != 8 {
            return Err(DecodeError::UnsupportedFeature);
        }
        self.height = u32::from(r.be16()?);
        self.width = u32::from(r.be16()?);
        pixel_len(self.width, self.height)?;
        self.progressive = progressive;
        let count = r.byte()? as usize;
        if !matches!(count, 1 | 3 | 4) {
            return Err(DecodeError::UnsupportedFeature);
        }
        let mut total = 0;
        for _ in 0..count {
            let id = r.byte()?;
            let sample = r.byte()?;
            let h = (sample >> 4) as usize;
            let v = (sample & 15) as usize;
            let q = r.byte()? as usize;
            if !(1..=4).contains(&h)
                || !(1..=4).contains(&v)
                || q > 3
                || self.components.iter().any(|c| c.id == id)
            {
                return Err(DecodeError::InvalidHeader);
            }
            self.max_h = self.max_h.max(h);
            self.max_v = self.max_v.max(v);
            total += h * v;
            self.components.push(Component {
                id,
                h,
                v,
                quant_id: q,
                blocks_w: 0,
                blocks_h: 0,
                actual_w: 0,
                actual_h: 0,
                coefficients: Vec::new(),
                quant: None,
                approximation: [-1; 64],
            });
        }
        if total > 10 || r.remaining() != 0 {
            return Err(DecodeError::InvalidHeader);
        }
        self.mcus_w = (self.width as usize).div_ceil(self.max_h * 8);
        self.mcus_h = (self.height as usize).div_ceil(self.max_v * 8);
        for c in &mut self.components {
            c.blocks_w = self.mcus_w * c.h;
            c.blocks_h = self.mcus_h * c.v;
            c.actual_w = (self.width as usize * c.h).div_ceil(self.max_h * 8);
            c.actual_h = (self.height as usize * c.v).div_ceil(self.max_v * 8);
            c.coefficients = budget.zeroed(c.blocks_w * c.blocks_h * 64)?;
        }
        Ok(())
    }

    fn scan(&mut self, s: &mut Reader<'_>, r: &mut Reader<'_>) -> Result<()> {
        // Sequential scans fill coefficients once; progressive scans transmit and refine them
        // over multiple passes.
        let count = s.byte()? as usize;
        if count == 0 || count > self.components.len() {
            return Err(DecodeError::InvalidData);
        }
        let mut selectors = [(0usize, 0usize, 0usize); 4];
        for n in 0..count {
            let id = s.byte()?;
            let index = self
                .components
                .iter()
                .position(|c| c.id == id)
                .ok_or(DecodeError::InvalidData)?;
            if selectors[..n].iter().any(|sel| sel.0 == index) {
                return Err(DecodeError::InvalidData);
            }
            let tables = s.byte()?;
            if tables >> 4 > 3 || tables & 15 > 3 {
                return Err(DecodeError::InvalidData);
            }
            selectors[n] = (index, (tables >> 4) as usize, (tables & 15) as usize);
        }
        let start = s.byte()? as usize;
        let end = s.byte()? as usize;
        let approx = s.byte()?;
        let high = approx >> 4;
        let low = approx & 15;
        if s.remaining() != 0 || start > end || end > 63 || high > 13 || low > 13 {
            return Err(DecodeError::InvalidData);
        }
        if self.progressive {
            if (start == 0 && end != 0)
                || (start != 0 && count != 1)
                || (high != 0 && high != low + 1)
            {
                return Err(DecodeError::InvalidData);
            }
        } else if start != 0 || end != 63 || approx != 0 {
            return Err(DecodeError::InvalidData);
        }
        for &(index, dc, ac) in &selectors[..count] {
            let c = &mut self.components[index];
            for a in &mut c.approximation[start..=end] {
                if (high == 0 && *a != -1) || (high != 0 && *a != high as i8) {
                    return Err(DecodeError::InvalidData);
                }
                *a = low as i8;
            }
            if c.quant.is_none() {
                c.quant = Some(self.quant[c.quant_id].ok_or(DecodeError::InvalidData)?);
            }
            if (start == 0 && high == 0 && self.dc[dc].is_none())
                || (end != 0 && self.ac[ac].is_none())
            {
                return Err(DecodeError::InvalidData);
            }
        }
        let first = &self.components[selectors[0].0];
        let (cols, rows) = if count == 1 {
            (first.actual_w, first.actual_h)
        } else {
            (self.mcus_w, self.mcus_h)
        };
        let mut bits = Bits {
            r,
            byte: 0,
            left: 0,
        };
        let mut predictors = [0i32; 4];
        let mut eob = 0u32;
        let mut restart_index = 0;
        for mcu in 0..cols * rows {
            if mcu != 0 && self.restart != 0 && mcu % self.restart == 0 {
                if eob != 0 {
                    return Err(DecodeError::InvalidData);
                }
                bits.align()?;
                if marker(bits.r)? != 0xd0 + restart_index {
                    return Err(DecodeError::InvalidData);
                }
                restart_index = (restart_index + 1) & 7;
                predictors.fill(0);
            }
            for &(index, dc, ac) in &selectors[..count] {
                let c = &mut self.components[index];
                let (bh, bv) = if count == 1 { (1, 1) } else { (c.h, c.v) };
                for y in 0..bv {
                    for x in 0..bh {
                        let block_x = (mcu % cols) * bh + x;
                        let block_y = (mcu / cols) * bv + y;
                        let offset = (block_y * c.blocks_w + block_x) * 64;
                        let block = &mut c.coefficients[offset..offset + 64];
                        if !self.progressive {
                            let size = self.dc[dc]
                                .as_ref()
                                .expect("DC table checked")
                                .symbol(&mut bits)?;
                            if size > 11 {
                                return Err(DecodeError::InvalidData);
                            }
                            predictors[index] = predictors[index]
                                .checked_add(bits.signed(size)?)
                                .filter(|v| v.abs() <= 65535)
                                .ok_or(DecodeError::InvalidData)?;
                            block[0] = predictors[index];
                            ac_first(
                                block,
                                &mut bits,
                                self.ac[ac].as_ref().expect("AC table checked"),
                                1,
                                63,
                                0,
                                &mut eob,
                                false,
                            )?;
                        } else if start == 0 {
                            if high == 0 {
                                let size = self.dc[dc]
                                    .as_ref()
                                    .expect("DC table checked")
                                    .symbol(&mut bits)?;
                                if size > 11 {
                                    return Err(DecodeError::InvalidData);
                                }
                                predictors[index] = predictors[index]
                                    .checked_add(bits.signed(size)?)
                                    .filter(|v| v.abs() <= 65535)
                                    .ok_or(DecodeError::InvalidData)?;
                                block[0] = predictors[index] << low;
                            } else {
                                block[0] |= (bits.get(1)? as i32) << low;
                            }
                        } else if high == 0 {
                            ac_first(
                                block,
                                &mut bits,
                                self.ac[ac].as_ref().expect("AC table checked"),
                                start,
                                end,
                                low,
                                &mut eob,
                                true,
                            )?;
                        } else {
                            ac_refine(
                                block,
                                &mut bits,
                                self.ac[ac].as_ref().expect("AC table checked"),
                                start,
                                end,
                                low,
                                &mut eob,
                            )?;
                        }
                    }
                }
            }
        }
        if eob != 0 {
            return Err(DecodeError::InvalidData);
        }
        bits.align()?;
        Ok(())
    }

    fn render(self, budget: &mut Budget) -> Result<Image> {
        // Transform component blocks, upsample chroma, convert to RGB, then apply EXIF orientation.
        let mut matrix = [[0f32; 8]; 8];
        for (u, row) in matrix.iter_mut().enumerate() {
            for (x, value) in row.iter_mut().enumerate() {
                *value = ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 16.0).cos()
                    * if u == 0 {
                        std::f32::consts::FRAC_1_SQRT_2 / 2.0
                    } else {
                        0.5
                    };
            }
        }
        let mut planes = Vec::new();
        for c in &self.components {
            let stride = c.blocks_w * 8;
            let mut plane = budget.zeroed::<u8>(stride * c.blocks_h * 8)?;
            let quant = c.quant.ok_or(DecodeError::InvalidData)?;
            for by in 0..c.blocks_h {
                for bx in 0..c.blocks_w {
                    let offset = (by * c.blocks_w + bx) * 64;
                    idct(
                        &c.coefficients[offset..offset + 64],
                        &quant,
                        &matrix,
                        &mut plane[(by * 8 * stride + bx * 8)..],
                        stride,
                    );
                }
            }
            planes.push(plane);
        }
        let mut pixels = budget.zeroed(pixel_len(self.width, self.height)?)?;
        let rgb = !self.jfif
            && (self.adobe == Some(0)
                || self
                    .components
                    .iter()
                    .map(|c| c.id)
                    .eq(b"RGB".iter().copied()));
        if self.adobe.is_some_and(|v| v > 2) {
            return Err(DecodeError::UnsupportedFeature);
        }
        for (y, row) in pixels.chunks_exact_mut(self.width as usize * 4).enumerate() {
            for (x, p) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let mut values = [0u8; 4];
                for (i, c) in self.components.iter().enumerate() {
                    values[i] = sample(
                        &planes[i],
                        c,
                        x,
                        y,
                        self.width as usize,
                        self.height as usize,
                        self.max_h,
                        self.max_v,
                    );
                }
                match self.components.len() {
                    1 => p.copy_from_slice(&[values[0], values[0], values[0], 255]),
                    3 => {
                        let color = if rgb {
                            [values[0], values[1], values[2]]
                        } else {
                            ycbcr(values)
                        };
                        p[..3].copy_from_slice(&color);
                        p[3] = 255;
                    }
                    4 => {
                        let color = if self.adobe == Some(2) {
                            ycbcr(values).map(|v| 255 - v)
                        } else if self.adobe.is_some() {
                            [values[0], values[1], values[2]]
                        } else {
                            [255 - values[0], 255 - values[1], 255 - values[2]]
                        };
                        let black = if self.adobe.is_some() {
                            values[3]
                        } else {
                            255 - values[3]
                        };
                        for k in 0..3 {
                            p[k] = ((u16::from(color[k]) * u16::from(black) + 127) / 255) as u8;
                        }
                        p[3] = 255;
                    }
                    _ => unreachable!("component count checked"),
                }
            }
        }
        let (width, height, pixels) =
            orient(self.width, self.height, pixels, self.orientation, budget)?;
        Ok(Image::still(Format::Jpeg, width, height, pixels))
    }
}

#[allow(clippy::too_many_arguments)]
fn ac_first(
    block: &mut [i32],
    bits: &mut Bits<'_, '_>,
    table: &Huffman,
    start: usize,
    end: usize,
    low: u8,
    eob: &mut u32,
    progressive: bool,
) -> Result<()> {
    if *eob > 0 {
        *eob -= 1;
        return Ok(());
    }
    let mut k = start;
    while k <= end {
        let value = table.symbol(bits)?;
        let run = (value >> 4) as usize;
        let size = value & 15;
        if size == 0 {
            if run == 15 {
                k += 16;
                if k > end + 1 {
                    return Err(DecodeError::InvalidData);
                }
            } else {
                if !progressive && run != 0 {
                    return Err(DecodeError::InvalidData);
                }
                *eob = (1 << run) + bits.get(run as u8)? - 1;
                break;
            }
        } else {
            k += run;
            if k > end || size > 10 {
                return Err(DecodeError::InvalidData);
            }
            block[ZIGZAG[k]] = bits.signed(size)? << low;
            k += 1;
        }
    }
    Ok(())
}

fn refine(value: &mut i32, bits: &mut Bits<'_, '_>, bit: i32) -> Result<()> {
    if bits.get(1)? != 0 && *value & bit == 0 {
        *value += if *value > 0 { bit } else { -bit };
    }
    Ok(())
}

fn ac_refine(
    block: &mut [i32],
    bits: &mut Bits<'_, '_>,
    table: &Huffman,
    start: usize,
    end: usize,
    low: u8,
    eob: &mut u32,
) -> Result<()> {
    let bit = 1 << low;
    let mut k = start;
    if *eob == 0 {
        while k <= end {
            let symbol = table.symbol(bits)?;
            let mut run = (symbol >> 4) as usize;
            let size = symbol & 15;
            let new = if size == 1 {
                if bits.get(1)? != 0 { bit } else { -bit }
            } else if size == 0 {
                if run < 15 {
                    *eob = (1 << run) + bits.get(run as u8)?;
                    break;
                }
                run = 16;
                0
            } else {
                return Err(DecodeError::InvalidData);
            };
            while k <= end {
                let value = &mut block[ZIGZAG[k]];
                if *value != 0 {
                    refine(value, bits, bit)?;
                } else {
                    if run == 0 {
                        break;
                    }
                    run -= 1;
                    if run == 0 && new == 0 {
                        k += 1;
                        break;
                    }
                }
                k += 1;
            }
            if run != 0 {
                return Err(DecodeError::InvalidData);
            }
            if new != 0 {
                if k > end {
                    return Err(DecodeError::InvalidData);
                }
                block[ZIGZAG[k]] = new;
                k += 1;
            }
        }
    }
    if *eob > 0 {
        for &index in &ZIGZAG[k..=end] {
            if block[index] != 0 {
                refine(&mut block[index], bits, bit)?;
            }
        }
        *eob -= 1;
    }
    Ok(())
}

fn idct(block: &[i32], quant: &[u16; 64], matrix: &[[f32; 8]; 8], dest: &mut [u8], stride: usize) {
    if block[1..].iter().all(|v| *v == 0) {
        let value = (block[0] as f32 * f32::from(quant[0]) / 8.0 + 128.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        for row in dest.chunks_mut(stride).take(8) {
            row[..8].fill(value);
        }
        return;
    }
    // Keep independent x values adjacent in the innermost loops so the optimizer
    // can schedule each row as one multiply-add kernel.
    let mut temp = [[0.0f32; 8]; 8];
    for v in 0..8 {
        for u in 0..8 {
            let coefficient = block[v * 8 + u] as f32 * f32::from(quant[v * 8 + u]);
            for x in 0..8 {
                temp[v][x] += coefficient * matrix[u][x];
            }
        }
    }

    let mut output = [[128.0f32; 8]; 8];
    for v in 0..8 {
        for y in 0..8 {
            let basis = matrix[v][y];
            for x in 0..8 {
                output[y][x] += temp[v][x] * basis;
            }
        }
    }
    for y in 0..8 {
        for x in 0..8 {
            dest[y * stride + x] = output[y][x].round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn sample(
    plane: &[u8],
    c: &Component,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    max_h: usize,
    max_v: usize,
) -> u8 {
    let stride = c.blocks_w * 8;
    if c.h == max_h && c.v == max_v {
        return plane[y * stride + x];
    }
    let sx = ((x as f32 + 0.5) * c.h as f32 / max_h as f32 - 0.5).max(0.0);
    let sy = ((y as f32 + 0.5) * c.v as f32 / max_v as f32 - 0.5).max(0.0);
    let max_x = (width * c.h).div_ceil(max_h) - 1;
    let max_y = (height * c.v).div_ceil(max_v) - 1;
    let x0 = (sx as usize).min(max_x);
    let x1 = (x0 + 1).min(max_x);
    let y0 = (sy as usize).min(max_y);
    let y1 = (y0 + 1).min(max_y);
    let fx = sx.fract();
    let fy = sy.fract();
    let top =
        f32::from(plane[y0 * stride + x0]) * (1.0 - fx) + f32::from(plane[y0 * stride + x1]) * fx;
    let bottom =
        f32::from(plane[y1 * stride + x0]) * (1.0 - fx) + f32::from(plane[y1 * stride + x1]) * fx;
    (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8
}

fn ycbcr(v: [u8; 4]) -> [u8; 3] {
    let y = i32::from(v[0]) * 65536;
    let cb = i32::from(v[1]) - 128;
    let cr = i32::from(v[2]) - 128;
    [
        (y + 91881 * cr + 32768) >> 16,
        (y - 22554 * cb - 46802 * cr + 32768) >> 16,
        (y + 116130 * cb + 32768) >> 16,
    ]
    .map(|v| v.clamp(0, 255) as u8)
}

fn exif_orientation(bytes: &[u8]) -> Option<u16> {
    let data = bytes.strip_prefix(b"Exif\0\0")?;
    let little = match data.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let short = |p: usize| -> Option<u16> {
        let a = data.get(p..p.checked_add(2)?)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(a)
        } else {
            u16::from_be_bytes(a)
        })
    };
    let long = |p: usize| -> Option<u32> {
        let a = data.get(p..p.checked_add(4)?)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(a)
        } else {
            u32::from_be_bytes(a)
        })
    };
    if short(2)? != 42 {
        return None;
    }
    let ifd = long(4)? as usize;
    let count = short(ifd)? as usize;
    for i in 0..count {
        let offset = ifd.checked_add(2 + i * 12)?;
        if short(offset)? == 0x112 && short(offset + 2)? == 3 && long(offset + 4)? == 1 {
            return short(offset + 8).filter(|v| (1..=8).contains(v));
        }
    }
    None
}

fn orient(
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    orientation: u16,
    budget: &mut Budget,
) -> Result<(u32, u32, Vec<u8>)> {
    if orientation == 1 {
        return Ok((width, height, pixels));
    }
    let (w, h) = if orientation >= 5 {
        (height, width)
    } else {
        (width, height)
    };
    let mut output = budget.zeroed(pixels.len())?;
    for y in 0..height {
        for x in 0..width {
            let (dx, dy) = match orientation {
                2 => (width - 1 - x, y),
                3 => (width - 1 - x, height - 1 - y),
                4 => (x, height - 1 - y),
                5 => (y, x),
                6 => (height - 1 - y, x),
                7 => (height - 1 - y, width - 1 - x),
                8 => (y, width - 1 - x),
                _ => (x, y),
            };
            let src = (y as usize * width as usize + x as usize) * 4;
            let dst = (dy as usize * w as usize + dx as usize) * 4;
            output[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
        }
    }
    Ok((w, h, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_applies_all_exif_orientations() {
        let jpeg = include_bytes!("../tests/fixtures/baseline.jpg");
        let base = decode(jpeg).expect("base JPEG");
        for little in [false, true] {
            for orientation in 1u16..=8 {
                let short = |v: u16| {
                    if little {
                        v.to_le_bytes()
                    } else {
                        v.to_be_bytes()
                    }
                };
                let long = |v: u32| {
                    if little {
                        v.to_le_bytes()
                    } else {
                        v.to_be_bytes()
                    }
                };
                let mut exif = b"Exif\0\0".to_vec();
                exif.extend_from_slice(if little { b"II" } else { b"MM" });
                exif.extend_from_slice(&short(42));
                exif.extend_from_slice(&long(8));
                exif.extend_from_slice(&short(1));
                exif.extend_from_slice(&short(0x112));
                exif.extend_from_slice(&short(3));
                exif.extend_from_slice(&long(1));
                exif.extend_from_slice(&short(orientation));
                exif.extend_from_slice(&short(0));
                exif.extend_from_slice(&long(0));
                let mut data = vec![255, 216, 255, 225];
                data.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
                data.extend_from_slice(&exif);
                data.extend_from_slice(&jpeg[2..]);
                let image = decode(&data).expect("oriented JPEG");
                let (w, h) = if orientation >= 5 { (13, 19) } else { (19, 13) };
                assert_eq!((image.width(), image.height()), (w, h));
                for y in 0..h {
                    for x in 0..w {
                        let (sx, sy) = match orientation {
                            1 => (x, y),
                            2 => (18 - x, y),
                            3 => (18 - x, 12 - y),
                            4 => (x, 12 - y),
                            5 => (y, x),
                            6 => (y, 12 - x),
                            7 => (18 - y, 12 - x),
                            8 => (18 - y, x),
                            _ => unreachable!(),
                        };
                        let src = ((sy * 19 + sx) * 4) as usize;
                        let dst = ((y * w + x) * 4) as usize;
                        assert_eq!(&image.pixels()[dst..dst + 4], &base.pixels()[src..src + 4]);
                    }
                }
            }
        }
    }
}
