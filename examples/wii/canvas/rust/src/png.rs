use alloc::vec;
use alloc::vec::Vec;

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const MAX_PIXELS: usize = 512 * 512;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum Error {
    InvalidData,
    Unsupported,
    TooLarge,
}

pub fn decode(data: &[u8]) -> Result<Image, Error> {
    if !data.starts_with(SIGNATURE) {
        return Err(Error::InvalidData);
    }

    let mut offset = SIGNATURE.len();
    let mut header = None;
    let mut palette = Vec::new();
    let mut transparency = Vec::new();
    let mut compressed = Vec::new();
    let mut saw_idat = false;
    let mut idat_ended = false;
    let mut saw_end = false;

    while offset < data.len() {
        let length = read_u32(data, &mut offset)? as usize;
        let kind = take(data, &mut offset, 4)?;
        if !kind.iter().all(u8::is_ascii_alphabetic) || kind[2] & 0x20 != 0 {
            return Err(Error::InvalidData);
        }
        let chunk = take(data, &mut offset, length)?;
        let expected_crc = read_u32(data, &mut offset)?;
        if crc32(kind.iter().chain(chunk).copied()) != expected_crc {
            return Err(Error::InvalidData);
        }

        match kind {
            b"IHDR" => {
                if header.is_some() || length != 13 || saw_idat || offset != SIGNATURE.len() + 25 {
                    return Err(Error::InvalidData);
                }
                let width = u32::from_be_bytes(chunk[0..4].try_into().unwrap()) as usize;
                let height = u32::from_be_bytes(chunk[4..8].try_into().unwrap()) as usize;
                let depth = chunk[8];
                let color = chunk[9];
                if width == 0
                    || height == 0
                    || width.checked_mul(height).ok_or(Error::TooLarge)? > MAX_PIXELS
                {
                    return Err(Error::TooLarge);
                }
                if depth != 8 || !matches!(color, 3 | 6) || chunk[10] != 0 || chunk[11] != 0 {
                    return Err(Error::Unsupported);
                }
                if chunk[12] != 0 {
                    return Err(Error::Unsupported);
                }
                header = Some((width, height, color));
            }
            b"PLTE" => {
                if header.is_none()
                    || !matches!(header, Some((_, _, 3 | 6)))
                    || saw_idat
                    || !palette.is_empty()
                    || chunk.is_empty()
                    || chunk.len() > 768
                    || chunk.len() % 3 != 0
                {
                    return Err(Error::InvalidData);
                }
                palette.extend_from_slice(chunk);
            }
            b"tRNS" => {
                if !matches!(header, Some((_, _, 3)))
                    || palette.is_empty()
                    || saw_idat
                    || !transparency.is_empty()
                    || chunk.is_empty()
                    || chunk.len() > 256
                {
                    return Err(Error::InvalidData);
                }
                transparency.extend_from_slice(chunk);
            }
            b"IDAT" => {
                if idat_ended || header.is_none() {
                    return Err(Error::InvalidData);
                }
                saw_idat = true;
                compressed.extend_from_slice(chunk);
            }
            b"IEND" => {
                if length != 0 || !saw_idat {
                    return Err(Error::InvalidData);
                }
                saw_end = true;
                break;
            }
            b"acTL" | b"fcTL" | b"fdAT" => return Err(Error::Unsupported),
            _ => {
                if saw_idat {
                    idat_ended = true;
                } else if kind[0] & 0x20 == 0 {
                    return Err(Error::Unsupported);
                }
            }
        }
        if saw_idat && kind != b"IDAT" {
            idat_ended = true;
        }
    }

    if !saw_end || offset != data.len() {
        return Err(Error::InvalidData);
    }
    let (width, height, color) = header.ok_or(Error::InvalidData)?;
    if color == 3
        && (palette.is_empty() || palette.len() % 3 != 0 || transparency.len() > palette.len() / 3)
    {
        return Err(Error::InvalidData);
    }
    let channels = if color == 6 { 4 } else { 1 };
    let row_bytes = width.checked_mul(channels).ok_or(Error::TooLarge)?;
    let decoded_len = row_bytes
        .checked_add(1)
        .and_then(|n| n.checked_mul(height))
        .ok_or(Error::TooLarge)?;
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&compressed, decoded_len)
        .map_err(|_| Error::InvalidData)?;
    if raw.len() != decoded_len {
        return Err(Error::InvalidData);
    }

    let mut rgba = vec![
        0;
        width
            .checked_mul(height)
            .and_then(|n| n.checked_mul(4))
            .ok_or(Error::TooLarge)?
    ];
    let mut previous = vec![0; row_bytes];
    for y in 0..height {
        let start = y * (row_bytes + 1);
        let filter = raw[start];
        let mut row = raw[start + 1..start + 1 + row_bytes].to_vec();
        unfilter(&mut row, &previous, channels, filter)?;
        let output = &mut rgba[y * width * 4..(y + 1) * width * 4];
        if color == 6 {
            output.copy_from_slice(&row);
        } else {
            let (pixels, _) = output.as_chunks_mut::<4>();
            for (index, pixel) in pixels.iter_mut().enumerate() {
                let palette_index = row[index] as usize;
                let rgb = palette
                    .get(palette_index * 3..palette_index * 3 + 3)
                    .ok_or(Error::InvalidData)?;
                pixel[..3].copy_from_slice(rgb);
                pixel[3] = transparency.get(palette_index).copied().unwrap_or(255);
            }
        }
        previous.copy_from_slice(&row);
    }
    Ok(Image {
        width,
        height,
        rgba,
    })
}

fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, Error> {
    let bytes = take(data, offset, 4)?;
    Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
}

fn take<'a>(data: &'a [u8], offset: &mut usize, length: usize) -> Result<&'a [u8], Error> {
    let end = offset.checked_add(length).ok_or(Error::InvalidData)?;
    let result = data.get(*offset..end).ok_or(Error::InvalidData)?;
    *offset = end;
    Ok(result)
}

fn crc32(bytes: impl Iterator<Item = u8>) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 0 {
                crc >> 1
            } else {
                (crc >> 1) ^ 0xedb8_8320
            };
        }
    }
    !crc
}

fn unfilter(
    row: &mut [u8],
    previous: &[u8],
    bytes_per_pixel: usize,
    filter: u8,
) -> Result<(), Error> {
    match filter {
        0 => {}
        1 => {
            for i in bytes_per_pixel..row.len() {
                row[i] = row[i].wrapping_add(row[i - bytes_per_pixel]);
            }
        }
        2 => {
            for (pixel, above) in row.iter_mut().zip(previous) {
                *pixel = pixel.wrapping_add(*above);
            }
        }
        3 => {
            for i in 0..row.len() {
                let left = if i >= bytes_per_pixel {
                    row[i - bytes_per_pixel]
                } else {
                    0
                };
                row[i] =
                    row[i].wrapping_add(((u16::from(left) + u16::from(previous[i])) / 2) as u8);
            }
        }
        4 => {
            for i in 0..row.len() {
                let left = if i >= bytes_per_pixel {
                    row[i - bytes_per_pixel]
                } else {
                    0
                };
                let above = previous[i];
                let upper_left = if i >= bytes_per_pixel {
                    previous[i - bytes_per_pixel]
                } else {
                    0
                };
                row[i] = row[i].wrapping_add(paeth(left, above, upper_left));
            }
        }
        _ => return Err(Error::InvalidData),
    }
    Ok(())
}

fn paeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let estimate = i16::from(left) + i16::from(above) - i16::from(upper_left);
    let dl = (estimate - i16::from(left)).abs();
    let da = (estimate - i16::from(above)).abs();
    let dul = (estimate - i16::from(upper_left)).abs();
    if dl <= da && dl <= dul {
        left
    } else if da <= dul {
        above
    } else {
        upper_left
    }
}
