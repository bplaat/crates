/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{Read, Write};

use crate::error::{Error, Result};

/// MySQL packet reader.
pub struct PacketReader {
    buffer: Vec<u8>,
    position: usize,
}

impl PacketReader {
    /// Create new packet reader from bytes.
    pub fn new(data: Vec<u8>) -> Self {
        PacketReader {
            buffer: data,
            position: 0,
        }
    }

    /// Read exact number of bytes.
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        if self.position + count > self.buffer.len() {
            return Err(Error::Protocol("Not enough data in packet".into()));
        }
        let result = self.buffer[self.position..self.position + count].to_vec();
        self.position += count;
        Ok(result)
    }

    /// Read a single byte.
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.position >= self.buffer.len() {
            return Err(Error::Protocol("Not enough data in packet".into()));
        }
        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(byte)
    }

    /// Read 2 bytes as little-endian u16.
    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read 3 bytes as little-endian u32.
    pub fn read_u24(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(3)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]))
    }

    /// Read 4 bytes as little-endian u32.
    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read 8 bytes as little-endian u64.
    pub fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read length-encoded integer.
    pub fn read_lenenc_int(&mut self) -> Result<u64> {
        let first = self.read_u8()?;
        match first {
            0xfb => Ok(0), // NULL
            0xfc => self.read_u16().map(|v| v as u64),
            0xfd => self.read_u24().map(|v| v as u64),
            0xfe => self.read_u64(),
            _ => Ok(first as u64),
        }
    }

    /// Read length-encoded string.
    pub fn read_lenenc_string(&mut self) -> Result<Vec<u8>> {
        let len = self.read_lenenc_int()?;
        self.read_bytes(len as usize)
    }

    /// Read null-terminated string.
    pub fn read_null_terminated_string(&mut self) -> Result<Vec<u8>> {
        let start = self.position;
        while self.position < self.buffer.len() && self.buffer[self.position] != 0 {
            self.position += 1;
        }
        if self.position >= self.buffer.len() {
            return Err(Error::Protocol("String not null-terminated".into()));
        }
        let result = self.buffer[start..self.position].to_vec();
        self.position += 1; // Skip null terminator
        Ok(result)
    }

    /// Check if all data has been read.
    pub fn is_empty(&self) -> bool {
        self.position >= self.buffer.len()
    }

    /// Get remaining bytes count.
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Get current position.
    pub fn pos(&self) -> usize {
        self.position
    }

    /// Set position.
    pub fn set_pos(&mut self, pos: usize) {
        self.position = pos;
    }
}

/// MySQL packet writer.
pub struct PacketWriter {
    buffer: Vec<u8>,
}

impl PacketWriter {
    /// Create new packet writer.
    pub fn new() -> Self {
        PacketWriter { buffer: Vec::new() }
    }

    /// Write a single byte.
    pub fn write_u8(&mut self, val: u8) {
        self.buffer.push(val);
    }

    /// Write 2 bytes as little-endian u16.
    pub fn write_u16(&mut self, val: u16) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    /// Write 3 bytes as little-endian u32.
    pub fn write_u24(&mut self, val: u32) {
        self.buffer.extend_from_slice(&val.to_le_bytes()[..3]);
    }

    /// Write 4 bytes as little-endian u32.
    pub fn write_u32(&mut self, val: u32) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    /// Write 8 bytes as little-endian u64.
    pub fn write_u64(&mut self, val: u64) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    /// Write bytes.
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Write null-terminated string.
    pub fn write_null_terminated_string(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.push(0);
    }

    /// Write length-encoded string.
    pub fn write_lenenc_string(&mut self, data: &[u8]) {
        self.write_lenenc_int(data.len() as u64);
        self.write_bytes(data);
    }

    /// Write length-encoded integer.
    pub fn write_lenenc_int(&mut self, val: u64) {
        match val {
            0..=0xfa => self.write_u8(val as u8),
            0xfb..=0xffff => {
                self.write_u8(0xfc);
                self.write_u16(val as u16);
            }
            0x10000..=0xffffff => {
                self.write_u8(0xfd);
                self.write_u24(val as u32);
            }
            _ => {
                self.write_u8(0xfe);
                self.write_u64(val);
            }
        }
    }

    /// Get buffer as bytes.
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Read a complete MySQL packet from stream.
pub fn read_packet<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut header = [0u8; 4];
    reader.read_exact(&mut header)?;

    let len = u32::from_le_bytes([header[0], header[1], header[2], 0]) as usize;
    let mut packet = vec![0u8; len];
    reader.read_exact(&mut packet)?;

    Ok(packet)
}

/// Write a complete MySQL packet to stream with sequence number.
pub fn write_packet<W: Write>(writer: &mut W, data: &[u8], seq_num: u8) -> Result<()> {
    let len = data.len() as u32;
    let header = [
        (len & 0xff) as u8,
        ((len >> 8) & 0xff) as u8,
        ((len >> 16) & 0xff) as u8,
        seq_num,
    ];
    writer.write_all(&header)?;
    writer.write_all(data)?;
    Ok(())
}
