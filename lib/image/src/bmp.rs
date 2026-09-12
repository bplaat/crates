/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{Budget, DecodeError, Format, Image, Reader, Result, pixel_len};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Compression {
    Rgb,
    Rle8,
    Rle4,
    Bitfields,
    AlphaBitfields,
}

impl Compression {
    const fn is_rle(self) -> bool {
        matches!(self, Self::Rle8 | Self::Rle4)
    }
}

impl TryFrom<u32> for Compression {
    type Error = DecodeError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::Rgb),
            1 => Ok(Self::Rle8),
            2 => Ok(Self::Rle4),
            3 => Ok(Self::Bitfields),
            6 => Ok(Self::AlphaBitfields),
            _ => Err(DecodeError::UnsupportedFeature),
        }
    }
}

#[derive(Clone, Copy)]
struct ChannelMasks([u32; 4]);

impl ChannelMasks {
    const fn defaults(depth: u16) -> Self {
        if depth == 16 {
            Self([0x7c00, 0x03e0, 0x001f, 0])
        } else {
            Self([0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0])
        }
    }

    fn read(
        depth: u16,
        compression: Compression,
        dib_len: usize,
        header: &mut Reader<'_>,
        file: &mut Reader<'_>,
    ) -> Result<Self> {
        let mut masks = Self::defaults(depth);
        if !matches!(
            compression,
            Compression::Bitfields | Compression::AlphaBitfields
        ) {
            return Ok(masks);
        }
        let has_alpha = dib_len >= 56 || compression == Compression::AlphaBitfields;
        if dib_len >= 52 {
            masks.read_values(header, has_alpha)?;
        } else {
            masks.read_values(file, has_alpha)?;
        }
        masks.validate(depth, compression)?;
        Ok(masks)
    }

    fn read_values(&mut self, source: &mut Reader<'_>, has_alpha: bool) -> Result<()> {
        for mask in &mut self.0[..3] {
            *mask = source.le32()?;
        }
        if has_alpha {
            self.0[3] = source.le32()?;
        }
        Ok(())
    }

    fn validate(self, depth: u16, compression: Compression) -> Result<()> {
        if self.0[..3].contains(&0)
            || (compression == Compression::AlphaBitfields && self.0[3] == 0)
        {
            return Err(DecodeError::InvalidHeader);
        }
        let mut used = 0;
        for mask in self.0 {
            if mask == 0 {
                continue;
            }
            let value = mask >> mask.trailing_zeros();
            if value & value.wrapping_add(1) != 0
                || used & mask != 0
                || (depth < 32 && mask >> depth != 0)
            {
                return Err(DecodeError::InvalidHeader);
            }
            used |= mask;
        }
        Ok(())
    }

    fn component(self, value: u32, channel: usize) -> u8 {
        let mask = self.0[channel];
        let shift = mask.trailing_zeros();
        let max = mask >> shift;
        ((u64::from((value & mask) >> shift) * 255 + u64::from(max / 2)) / u64::from(max)) as u8
    }
}

struct BmpHeader {
    file_size: usize,
    pixel_offset: usize,
    dib_len: usize,
    width: u32,
    height: u32,
    top_down: bool,
    depth: u16,
    compression: Compression,
    #[cfg(feature = "ico")]
    image_size: usize,
    colors: usize,
    masks: ChannelMasks,
    pixel_len: usize,
}

impl BmpHeader {
    fn parse<'a>(data: &'a [u8]) -> Result<(Self, Reader<'a>)> {
        let mut r = Reader::new(data);
        r.take(2)?;
        let file_size = r.le32()? as usize;
        r.take(4)?;
        let pixel_offset = r.le32()? as usize;
        let mut header = Self::parse_dib(&mut r, false, 0)?;
        header.file_size = file_size;
        header.pixel_offset = pixel_offset;
        Ok((header, r))
    }

    fn parse_dib(r: &mut Reader<'_>, icon: bool, icon_colors: usize) -> Result<Self> {
        // CORE and INFO-family headers arrange palettes and masks differently.
        let dib_len = r.le32()? as usize;
        if !matches!(dib_len, 12 | 40 | 52 | 56 | 108 | 124) {
            return Err(DecodeError::UnsupportedFeature);
        }
        let mut h = Reader::new(r.take(dib_len - 4)?);
        let (width, mut height, top_down) = if dib_len == 12 {
            (u32::from(h.le16()?), u32::from(h.le16()?), false)
        } else {
            let w = h.le32()? as i32;
            let signed_h = h.le32()? as i32;
            if w <= 0 || signed_h == i32::MIN {
                return Err(DecodeError::InvalidHeader);
            }
            (w as u32, signed_h.unsigned_abs(), signed_h < 0)
        };
        if icon {
            if height % 2 != 0 {
                return Err(DecodeError::InvalidHeader);
            }
            height /= 2;
        }
        let pixel_len = pixel_len(width, height)?;
        if h.le16()? != 1 {
            return Err(DecodeError::InvalidHeader);
        }
        let depth = h.le16()?;
        if !matches!(depth, 1 | 4 | 8 | 16 | 24 | 32) {
            return Err(DecodeError::UnsupportedFeature);
        }
        let mut compression = Compression::Rgb;
        let mut _image_size = 0;
        let mut colors = 0;
        if dib_len != 12 {
            compression = Compression::try_from(h.le32()?)?;
            _image_size = h.le32()? as usize;
            h.take(8)?; // Pixels per meter.
            colors = h.le32()? as usize;
            h.le32()?;
        }
        if icon && colors == 0 && icon_colors != 0 {
            colors = icon_colors;
        }
        if (compression == Compression::Rle8 && depth != 8)
            || (compression == Compression::Rle4 && depth != 4)
            || (matches!(
                compression,
                Compression::Bitfields | Compression::AlphaBitfields
            ) && !matches!(depth, 16 | 32))
            || (top_down && compression.is_rle())
        {
            return Err(DecodeError::InvalidHeader);
        }
        let mut masks = ChannelMasks::read(depth, compression, dib_len, &mut h, r)?;
        if icon && depth == 32 && compression == Compression::Rgb {
            masks.0[3] = 0xff00_0000;
        }
        Ok(Self {
            file_size: 0,
            pixel_offset: 0,
            dib_len,
            width,
            height,
            top_down,
            depth,
            compression,
            #[cfg(feature = "ico")]
            image_size: _image_size,
            colors,
            masks,
            pixel_len,
        })
    }
}

pub(super) fn decode(data: &[u8]) -> Result<Image> {
    let (header, mut r) = BmpHeader::parse(data)?;
    let (palette, palette_len) = read_palette(&mut r, &header)?;
    if header.pixel_offset < r.pos
        || header.pixel_offset > data.len()
        || (header.file_size != 0
            && (header.file_size > data.len() || header.file_size < header.pixel_offset))
    {
        return Err(DecodeError::InvalidData);
    }
    let end = if header.file_size == 0 {
        data.len()
    } else {
        header.file_size
    };
    let mut budget = Budget::default();
    let (pixels, _) = decode_pixels(
        &data[header.pixel_offset..end],
        &header,
        &palette[..palette_len],
        &mut budget,
    )?;
    Ok(Image::still(
        Format::Bmp,
        header.width,
        header.height,
        pixels,
    ))
}

#[cfg(feature = "ico")]
pub(super) fn decode_icon(
    data: &[u8],
    directory_width: u32,
    directory_height: u32,
    directory_colors: usize,
) -> Result<Image> {
    let mut r = Reader::new(data);
    let header = BmpHeader::parse_dib(&mut r, true, directory_colors)?;
    if header.width != directory_width || header.height != directory_height {
        return Err(DecodeError::InvalidHeader);
    }
    let (palette, palette_len) = read_palette(&mut r, &header)?;
    let pixel_start = r.pos;
    let mut budget = Budget::default();
    let (mut pixels, consumed) = decode_pixels(
        &data[pixel_start..],
        &header,
        &palette[..palette_len],
        &mut budget,
    )?;
    let xor_len = if header.compression.is_rle() && header.image_size != 0 {
        if header.image_size < consumed {
            return Err(DecodeError::InvalidData);
        }
        header.image_size
    } else {
        consumed
    };
    let mask = data
        .get(
            pixel_start
                .checked_add(xor_len)
                .ok_or(DecodeError::InvalidData)?..,
        )
        .ok_or(DecodeError::InvalidData)?;
    apply_icon_mask(mask, &header, &mut pixels)?;
    Ok(Image::still(
        Format::Ico,
        header.width,
        header.height,
        pixels,
    ))
}

fn read_palette(r: &mut Reader<'_>, header: &BmpHeader) -> Result<([[u8; 4]; 256], usize)> {
    let mut palette = [[0u8, 0, 0, 255]; 256];
    let palette_len = if header.depth <= 8 {
        let max = 1usize << header.depth;
        if header.colors > max {
            return Err(DecodeError::InvalidHeader);
        }
        if header.colors == 0 {
            max
        } else {
            header.colors
        }
    } else {
        0
    };
    for color in &mut palette[..palette_len] {
        let bgr = r.take(if header.dib_len == 12 { 3 } else { 4 })?;
        color[..3].copy_from_slice(&[bgr[2], bgr[1], bgr[0]]);
    }
    Ok((palette, palette_len))
}

fn decode_pixels(
    data: &[u8],
    header: &BmpHeader,
    palette: &[[u8; 4]],
    budget: &mut Budget,
) -> Result<(Vec<u8>, usize)> {
    let mut r = Reader::new(data);
    let mut pixels = budget.zeroed(header.pixel_len)?;
    if header.compression.is_rle() {
        pixels.as_chunks_mut::<4>().0.fill(palette[0]);
        rle(
            &mut r,
            &mut pixels,
            header.width as usize,
            header.height as usize,
            header.depth,
            palette,
        )?;
    } else {
        let stride =
            usize::try_from((u64::from(header.width) * u64::from(header.depth)).div_ceil(32) * 4)
                .map_err(|_| DecodeError::ImageTooLarge)?;
        if r.remaining() < stride * header.height as usize {
            return Err(DecodeError::InvalidData);
        }
        for y in 0..header.height as usize {
            let row = r.take(stride)?;
            let dest_y = if header.top_down {
                y
            } else {
                header.height as usize - 1 - y
            };
            let dest = &mut pixels
                [dest_y * header.width as usize * 4..(dest_y + 1) * header.width as usize * 4];
            let dest = dest.as_chunks_mut::<4>().0;
            match header.depth {
                1 | 4 | 8 => {
                    for (x, pixel) in dest.iter_mut().enumerate() {
                        let bit = x * header.depth as usize;
                        let index = (row[bit / 8] >> (8 - header.depth as usize - bit % 8))
                            & ((1u16 << header.depth) - 1) as u8;
                        pixel.copy_from_slice(
                            palette
                                .get(index as usize)
                                .ok_or(DecodeError::InvalidData)?,
                        );
                    }
                }
                24 => {
                    for (pixel, s) in dest.iter_mut().zip(row.as_chunks::<3>().0) {
                        pixel.copy_from_slice(&[s[2], s[1], s[0], 255]);
                    }
                }
                _ => {
                    for (x, pixel) in dest.iter_mut().enumerate() {
                        let v = if header.depth == 16 {
                            u32::from(u16::from_le_bytes(
                                row[x * 2..x * 2 + 2].try_into().expect("two bytes"),
                            ))
                        } else {
                            u32::from_le_bytes(
                                row[x * 4..x * 4 + 4].try_into().expect("four bytes"),
                            )
                        };
                        for (channel, value) in pixel[..3].iter_mut().enumerate() {
                            *value = header.masks.component(v, channel);
                        }
                        pixel[3] = if header.masks.0[3] == 0 {
                            255
                        } else {
                            header.masks.component(v, 3)
                        };
                    }
                }
            }
        }
    }
    Ok((pixels, r.pos))
}

#[cfg(feature = "ico")]
fn apply_icon_mask(mask: &[u8], header: &BmpHeader, pixels: &mut [u8]) -> Result<()> {
    let has_alpha = header.depth == 32
        && header.masks.0[3] != 0
        && pixels.as_chunks::<4>().0.iter().any(|p| p[3] != 0);
    if header.depth == 32 && !has_alpha {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
    }
    if has_alpha {
        return Ok(());
    }
    let stride = usize::try_from(u64::from(header.width).div_ceil(32) * 4)
        .map_err(|_| DecodeError::ImageTooLarge)?;
    let len = stride
        .checked_mul(header.height as usize)
        .ok_or(DecodeError::ImageTooLarge)?;
    if mask.len() < len {
        return Ok(());
    }
    for source_y in 0..header.height as usize {
        let row = &mask[source_y * stride..(source_y + 1) * stride];
        let y = if header.top_down {
            source_y
        } else {
            header.height as usize - 1 - source_y
        };
        for x in 0..header.width as usize {
            if row[x / 8] & (0x80 >> (x % 8)) != 0 {
                pixels[(y * header.width as usize + x) * 4 + 3] = 0;
            }
        }
    }
    Ok(())
}

fn rle(
    r: &mut Reader<'_>,
    pixels: &mut [u8],
    width: usize,
    height: usize,
    depth: u16,
    palette: &[[u8; 4]],
) -> Result<()> {
    // RLE4/RLE8 alternate encoded runs with escape commands for literals, row ends, and deltas.
    let mut x = 0;
    let mut y = 0;
    loop {
        let count = r.byte()? as usize;
        let value = r.byte()?;
        if count == 0 {
            match value {
                0 => {
                    x = 0;
                    y += 1;
                    if y > height {
                        return Err(DecodeError::InvalidData);
                    }
                }
                1 => return Ok(()),
                2 => {
                    x += r.byte()? as usize;
                    y += r.byte()? as usize;
                    if x > width || y >= height {
                        return Err(DecodeError::InvalidData);
                    }
                }
                _ => {
                    let count = value as usize;
                    if y >= height || count > width - x {
                        return Err(DecodeError::InvalidData);
                    }
                    let size = if depth == 8 { count } else { count.div_ceil(2) };
                    let bytes = r.take(size)?;
                    for i in 0..count {
                        let index = if depth == 8 {
                            bytes[i]
                        } else {
                            (bytes[i / 2] >> (4 - i % 2 * 4)) & 15
                        };
                        put(
                            pixels,
                            ((height - 1 - y) * width + x + i) * 4,
                            index,
                            palette,
                        )?;
                    }
                    x += count;
                    if size % 2 != 0 {
                        r.byte()?;
                    }
                }
            }
        } else {
            if y >= height || count > width - x {
                return Err(DecodeError::InvalidData);
            }
            for i in 0..count {
                let index = if depth == 8 {
                    value
                } else {
                    (value >> (4 - i % 2 * 4)) & 15
                };
                put(
                    pixels,
                    ((height - 1 - y) * width + x + i) * 4,
                    index,
                    palette,
                )?;
            }
            x += count;
        }
    }
}

fn put(pixels: &mut [u8], offset: usize, index: u8, palette: &[[u8; 4]]) -> Result<()> {
    pixels[offset..offset + 4].copy_from_slice(
        palette
            .get(index as usize)
            .ok_or(DecodeError::InvalidData)?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_values_are_validated() {
        assert!(matches!(
            Compression::try_from(6),
            Ok(Compression::AlphaBitfields)
        ));
        assert!(Compression::try_from(4).is_err());
    }

    fn bitmap(
        width: i32,
        height: i32,
        depth: u16,
        compression: u32,
        masks: &[u32],
        palette: &[[u8; 4]],
        pixels: &[u8],
    ) -> Vec<u8> {
        let offset = 54 + masks.len() * 4 + palette.len() * 4;
        let mut out = b"BM".to_vec();
        out.extend_from_slice(&((offset + pixels.len()) as u32).to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&depth.to_le_bytes());
        out.extend_from_slice(&compression.to_le_bytes());
        out.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        out.extend_from_slice(&[0; 8]);
        out.extend_from_slice(&(palette.len() as u32).to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        for m in masks {
            out.extend_from_slice(&m.to_le_bytes());
        }
        for p in palette {
            out.extend_from_slice(p);
        }
        out.extend_from_slice(pixels);
        out
    }

    #[test]
    fn bmp_depths_bitfields_top_down_and_core_header() {
        let colors = [[0, 0, 255, 0], [0, 255, 0, 0]];
        for depth in [1, 4, 8] {
            let rows = match depth {
                1 => vec![0x40, 0, 0, 0],
                4 => vec![0x01, 0, 0, 0],
                _ => vec![0, 1, 0, 0],
            };
            assert_eq!(
                decode(&bitmap(2, 1, depth, 0, &[], &colors, &rows))
                    .expect("indexed BMP")
                    .pixels(),
                &[255, 0, 0, 255, 0, 255, 0, 255]
            );
        }
        assert_eq!(
            decode(&bitmap(
                1,
                -2,
                24,
                0,
                &[],
                &[],
                &[0, 0, 255, 0, 0, 255, 0, 0]
            ))
            .expect("top down")
            .pixels(),
            &[255, 0, 0, 255, 0, 255, 0, 255]
        );
        assert_eq!(
            decode(&bitmap(1, 1, 16, 0, &[], &[], &[0, 0x7c, 0, 0]))
                .expect("RGB555")
                .pixels(),
            &[255, 0, 0, 255]
        );
        assert_eq!(
            decode(&bitmap(
                1,
                1,
                16,
                3,
                &[0xf800, 0x07e0, 0x001f],
                &[],
                &[0xe0, 0x07, 0, 0]
            ))
            .expect("RGB565")
            .pixels(),
            &[0, 255, 0, 255]
        );
        assert_eq!(
            decode(&bitmap(
                1,
                1,
                32,
                6,
                &[0xff0000, 0xff00, 0xff, 0xff000000],
                &[],
                &[0, 0, 255, 128]
            ))
            .expect("alpha bitfields")
            .pixels(),
            &[255, 0, 0, 128]
        );
        assert_eq!(
            decode(&bitmap(1, 1, 32, 0, &[], &[], &[0, 0, 255, 0]))
                .expect("unused alpha")
                .pixels(),
            &[255, 0, 0, 255]
        );
        assert!(
            decode(&bitmap(
                1,
                1,
                16,
                3,
                &[0xf800, 0xf800, 0x001f],
                &[],
                &[0; 4]
            ))
            .is_err()
        );
        let mut core = b"BM".to_vec();
        core.extend_from_slice(&30u32.to_le_bytes());
        core.extend_from_slice(&[0; 4]);
        core.extend_from_slice(&26u32.to_le_bytes());
        core.extend_from_slice(&12u32.to_le_bytes());
        for v in [1u16, 1, 1, 24] {
            core.extend_from_slice(&v.to_le_bytes());
        }
        core.extend_from_slice(&[0, 0, 255, 0]);
        assert_eq!(
            decode(&core).expect("CORE bitmap").pixels(),
            &[255, 0, 0, 255]
        );
    }

    #[test]
    fn bmp_rle_runs_literals_deltas_and_padding() {
        let palette = [[0, 0, 0, 0], [0, 0, 255, 0], [0, 255, 0, 0], [255, 0, 0, 0]];
        for (depth, compression, bytes) in [
            (8, 1, vec![0, 3, 1, 2, 3, 0, 0, 0, 0, 2, 1, 0, 2, 1, 0, 1]),
            (
                4,
                2,
                vec![0, 3, 0x12, 0x30, 0, 0, 0, 2, 1, 0, 2, 0x11, 0, 1],
            ),
        ] {
            let image = decode(&bitmap(3, 2, depth, compression, &[], &palette, &bytes))
                .expect("RLE bitmap");
            assert_eq!(
                image.pixels(),
                &[
                    0, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255,
                    0, 0, 255, 255
                ]
            );
        }
        assert!(decode(&bitmap(1, 1, 8, 1, &[], &palette, &[2, 1, 0, 1])).is_err());
    }

    #[test]
    fn bmp_v4_v5_embedded_masks() {
        for header_len in [108u32, 124] {
            let mut data = bitmap(
                1,
                1,
                32,
                3,
                &[0xff0000, 0xff00, 0xff, 0xff000000],
                &[],
                &[1, 2, 3, 128],
            );
            data.splice(70..70, std::iter::repeat_n(0, (header_len - 56) as usize));
            let len = data.len() as u32;
            data[2..6].copy_from_slice(&len.to_le_bytes());
            data[10..14].copy_from_slice(&(14 + header_len).to_le_bytes());
            data[14..18].copy_from_slice(&header_len.to_le_bytes());
            assert_eq!(
                decode(&data).expect("V4/V5 bitmap").pixels(),
                &[3, 2, 1, 128]
            );
        }
    }
}
