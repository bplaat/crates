/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Decode a single gzip member into memory.

const MAX_OUTPUT_SIZE: usize = 1024 * 1024 * 1024;

const fn crc32_table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32_TABLE: [u32; 256] = crc32_table();

fn crc32(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in data {
        crc = (crc >> 8) ^ CRC32_TABLE[((crc as u8) ^ byte) as usize];
    }
    !crc
}

fn skip_c_string(data: &[u8], offset: &mut usize) -> Option<()> {
    *offset += data.get(*offset..)?.iter().position(|&byte| byte == 0)? + 1;
    Some(())
}

/// Compress bytes into a single gzip member.
pub fn compress(data: &[u8]) -> Vec<u8> {
    let payload = miniz_oxide::deflate::compress_to_vec(data, 6);
    let mut output = Vec::with_capacity(18 + payload.len());
    output.extend_from_slice(&[0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 255]);
    output.extend_from_slice(&payload);
    output.extend_from_slice(&crc32(data).to_le_bytes());
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    output
}

/// Decompress a single gzip member, returning `None` for invalid data or output over 1 GiB.
pub fn decompress(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 18 || data[..3] != [0x1f, 0x8b, 0x08] || data[3] & 0xe0 != 0 {
        return None;
    }
    let flags = data[3];
    let footer = data.len() - 8;
    let mut offset = 10usize;
    if flags & 0x04 != 0 {
        let xlen = u16::from_le_bytes(data.get(offset..offset + 2)?.try_into().ok()?) as usize;
        offset = offset.checked_add(2 + xlen)?;
    }
    if flags & 0x08 != 0 {
        skip_c_string(data.get(..footer)?, &mut offset)?;
    }
    if flags & 0x10 != 0 {
        skip_c_string(data.get(..footer)?, &mut offset)?;
    }
    if flags & 0x02 != 0 {
        let expected = u16::from_le_bytes(data.get(offset..offset + 2)?.try_into().ok()?);
        if crc32(data.get(..offset)?) as u16 != expected {
            return None;
        }
        offset += 2;
    }
    let compressed = data.get(offset..footer)?;
    let output =
        miniz_oxide::inflate::decompress_to_vec_with_limit(compressed, MAX_OUTPUT_SIZE).ok()?;
    let expected_crc = u32::from_le_bytes(data.get(footer..footer + 4)?.try_into().ok()?);
    let expected_size = u32::from_le_bytes(data.get(footer + 4..)?.try_into().ok()?);
    (crc32(&output) == expected_crc && output.len() as u32 == expected_size).then_some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &[u8] = include_bytes!("../tests/fixtures/small.gz");

    #[test]
    fn decompresses_gzip() {
        assert_eq!(decompress(SMALL).as_deref(), Some(&b"hello world"[..]));
    }

    #[test]
    fn roundtrips_gzip() {
        for size in [0, 1, 1024, 1024 * 1024] {
            let data = (0..size).map(|i| i as u8).collect::<Vec<_>>();
            assert_eq!(decompress(&compress(&data)), Some(data));
        }
    }

    #[test]
    fn rejects_invalid_header_and_trailer() {
        let mut data = SMALL.to_vec();
        data[3] = 0x20;
        assert!(decompress(&data).is_none());
        data[3] = 0;
        let footer = data.len() - 8;
        data[footer] ^= 1;
        assert!(decompress(&data).is_none());
        assert!(decompress(&SMALL[..SMALL.len() - 1]).is_none());
    }

    #[test]
    fn handles_optional_header_fields() {
        let mut data = SMALL.to_vec();
        data[3] = 0x1c;
        data.splice(10..10, [2, 0, 1, 2, b'a', 0, b'b', 0]);
        assert_eq!(decompress(&data).as_deref(), Some(&b"hello world"[..]));
        data[3] |= 0x02;
        let header_crc = crc32(&data[..18]) as u16;
        data.splice(18..18, header_crc.to_le_bytes());
        assert_eq!(decompress(&data).as_deref(), Some(&b"hello world"[..]));
        data[18] ^= 1;
        assert!(decompress(&data).is_none());
    }
}
