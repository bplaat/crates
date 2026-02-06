/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;

use sha1::Digest;
use pbkdf2::Sha256;

use crate::error::{Error, Result};
use crate::protocol::{PacketReader, PacketWriter, read_packet, write_packet};
use crate::types::CapabilityFlags;
use crate::value::{Row, Value};

/// MySQL connection.
pub struct Connection {
    stream: BufStream,
    seq_num: u8,
}

struct BufStream {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl BufStream {
    fn new(stream: TcpStream) -> Self {
        let stream_clone = stream.try_clone().expect("Failed to clone TcpStream");
        BufStream {
            reader: BufReader::new(stream),
            writer: BufWriter::new(stream_clone),
        }
    }

    fn read_packet(&mut self) -> Result<Vec<u8>> {
        read_packet(&mut self.reader)
    }

    fn write_packet(&mut self, data: &[u8], seq_num: u8) -> Result<()> {
        write_packet(&mut self.writer, data, seq_num)?;
        self.writer.flush()?;
        Ok(())
    }
}

impl Connection {
    /// Connect to MySQL server.
    pub fn connect(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self> {
        let stream = TcpStream::connect((host, port))
            .map_err(|e| Error::Connection(format!("Failed to connect to {host}:{port}: {e}")))?;

        let mut conn = Connection {
            stream: BufStream::new(stream),
            seq_num: 0,
        };

        conn.handshake(user, password, database)?;
        Ok(conn)
    }

    fn handshake(&mut self, user: &str, password: &str, database: &str) -> Result<()> {
        // Read initial handshake packet
        let packet = self.stream.read_packet()?;
        let mut reader = PacketReader::new(packet);

        let protocol_version = reader.read_u8()?;
        if protocol_version != 10 {
            return Err(Error::Protocol(format!(
                "Unsupported protocol version: {protocol_version}",
            )));
        }

        let server_version = reader.read_null_terminated_string()?;
        let _version_str = String::from_utf8_lossy(&server_version);

        let _connection_id = reader.read_u32()?;
        let auth_plugin_data_1 = reader.read_bytes(8)?;

        reader.read_u8()?; // filler
        let capability_flags_1 = reader.read_u16()?;
        let charset = reader.read_u8()?;
        let _status_flags = reader.read_u16()?;
        let capability_flags_2 = reader.read_u16()?;

        let capability_flags =
            CapabilityFlags::new((capability_flags_1 as u32) | ((capability_flags_2 as u32) << 16));

        let auth_plugin_data_len = reader.read_u8()? as usize;
        reader.read_bytes(10)?; // reserved

        let auth_plugin_data_2_len = if auth_plugin_data_len > 8 {
            auth_plugin_data_len - 8
        } else {
            13
        };

        let mut auth_plugin_data = auth_plugin_data_1.clone();
        if auth_plugin_data_2_len > 0 {
            let remaining = reader.read_bytes(auth_plugin_data_2_len)?;
            auth_plugin_data.extend(remaining);
        }

        let auth_plugin_name = if capability_flags.supports_plugin_auth() {
            String::from_utf8_lossy(&reader.read_null_terminated_string()?).to_string()
        } else {
            "mysql_native_password".to_string()
        };

        // Compute authentication response
        let auth_response = if auth_plugin_name == "caching_sha2_password" {
            compute_caching_sha2_password(&auth_plugin_data, password)?
        } else {
            compute_mysql_native_password(&auth_plugin_data, password)?
        };

        // Send handshake response
        self.seq_num = 1;
        let mut response = PacketWriter::new();

        let mut client_flags = 0u32;
        client_flags |= 0x00000001; // CLIENT_LONG_PASSWORD
        client_flags |= 0x00000002; // CLIENT_FOUND_ROWS
        client_flags |= 0x00000004; // CLIENT_LONG_FLAG
        client_flags |= 0x00000008; // CLIENT_CONNECT_WITH_DB
        client_flags |= 0x00000020; // CLIENT_IGNORE_SPACE
        client_flags |= 0x00000080; // CLIENT_INTERACTIVE
        client_flags |= 0x00000200; // CLIENT_LOCAL_FILES
        client_flags |= 0x00000800; // CLIENT_IGNORE_SIGPIPE
        client_flags |= 0x00008000; // CLIENT_SECURE_CONNECTION
        client_flags |= 0x00020000; // CLIENT_MULTI_STATEMENTS
        client_flags |= 0x00040000; // CLIENT_MULTI_RESULTS

        if capability_flags.supports_plugin_auth() {
            client_flags |= 0x00080000; // CLIENT_PLUGIN_AUTH
        }

        response.write_u32(client_flags);
        response.write_u32(16 * 1024 * 1024); // max_allowed_packet
        response.write_u8(charset);
        response.write_bytes(&[0u8; 23]); // reserved

        response.write_null_terminated_string(user);

        if auth_plugin_name == "caching_sha2_password" && !password.is_empty() {
            response.write_u8(auth_response.len() as u8);
            response.write_bytes(&auth_response);
        } else {
            response.write_lenenc_string(&auth_response);
        }

        if !database.is_empty() {
            response.write_null_terminated_string(database);
        }

        if capability_flags.supports_plugin_auth() {
            response.write_null_terminated_string(&auth_plugin_name);
        }

        let payload = response.finish();
        self.write_packet(&payload)?;

        // Read response
        let response_packet = self.stream.read_packet()?;
        self.handle_response(&response_packet)?;

        Ok(())
    }

    fn write_packet(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_packet(data, self.seq_num)?;
        self.seq_num = self.seq_num.wrapping_add(1);
        Ok(())
    }

    fn handle_response(&mut self, packet: &[u8]) -> Result<()> {
        if packet.is_empty() {
            return Err(Error::Protocol("Empty response packet".into()));
        }

        match packet[0] {
            0x00 => Ok(()), // OK packet
            0xff => {
                // Error packet
                let mut reader = PacketReader::new(packet[1..].to_vec());
                let error_code = reader.read_u16()?;
                reader.read_u8()?; // SQL state marker
                reader.read_bytes(5)?; // SQL state
                let error_msg =
                    String::from_utf8_lossy(&reader.read_bytes(reader.remaining())?).to_string();
                Err(Error::Server {
                    code: error_code,
                    message: error_msg,
                })
            }
            _ => Err(Error::Protocol("Unexpected response packet type".into())),
        }
    }

    /// Execute a query and return rows.
    pub fn query(&mut self, sql: &str) -> Result<Vec<Row>> {
        let mut packet = PacketWriter::new();
        packet.write_u8(0x03); // COM_QUERY
        packet.write_bytes(sql.as_bytes());

        self.write_packet(&packet.finish())?;

        self.read_result()
    }

    fn read_result(&mut self) -> Result<Vec<Row>> {
        let packet = self.stream.read_packet()?;

        if packet.is_empty() {
            return Err(Error::Protocol("Empty result packet".into()));
        }

        match packet[0] {
            0x00 => {
                // OK packet - no result set
                Ok(Vec::new())
            }
            0xff => {
                // Error packet
                let mut reader = PacketReader::new(packet[1..].to_vec());
                let error_code = reader.read_u16()?;
                reader.read_u8()?;
                reader.read_bytes(5)?;
                let error_msg =
                    String::from_utf8_lossy(&reader.read_bytes(reader.remaining())?).to_string();
                Err(Error::Server {
                    code: error_code,
                    message: error_msg,
                })
            }
            _ => {
                // Result set
                let mut reader = PacketReader::new(packet);
                let column_count = reader.read_lenenc_int()? as usize;

                // Read column definitions
                for _ in 0..column_count {
                    let col_packet = self.stream.read_packet()?;
                    // Parse column definition but don't store for now
                    let _ = col_packet;
                }

                // Read EOF after column definitions
                let _eof = self.stream.read_packet()?;

                // Read rows
                let mut rows = Vec::new();
                loop {
                    let row_packet = self.stream.read_packet()?;
                    if row_packet.is_empty() {
                        break;
                    }

                    if row_packet[0] == 0xfe && row_packet.len() < 9 {
                        // EOF packet
                        break;
                    }

                    let row = self.parse_row(&row_packet, column_count)?;
                    rows.push(row);
                }

                Ok(rows)
            }
        }
    }

    fn parse_row(&self, packet: &[u8], column_count: usize) -> Result<Row> {
        let mut reader = PacketReader::new(packet.to_vec());
        let mut row = Row::new();

        for _ in 0..column_count {
            let value_len = reader.read_lenenc_int()?;
            if value_len == 0 && reader.remaining() > 0 {
                // Check if next byte indicates NULL
                let peek = reader.read_u8()?;
                if peek == 0xfb {
                    row.push(Value::Null);
                } else {
                    // Push the byte back by re-reading length-encoded value
                    reader.set_pos(reader.pos() - 1);
                    let bytes = reader.read_lenenc_string()?;
                    row.push(Value::Bytes(bytes));
                }
            } else if value_len == 0xfb_u64 {
                row.push(Value::Null);
            } else {
                let bytes = reader.read_bytes(value_len as usize)?;
                row.push(Value::Bytes(bytes));
            }
        }

        Ok(row)
    }
}

fn compute_mysql_native_password(auth_data: &[u8], password: &str) -> Result<Vec<u8>> {
    if password.is_empty() {
        return Ok(Vec::new());
    }

    let mut hasher = sha1::Sha1::new();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();

    let mut hasher2 = sha1::Sha1::new();
    hasher2.update(&password_hash[..]);
    hasher2.update(&auth_data[..20]);
    let final_hash = hasher2.finalize();

    let mut response = Vec::with_capacity(20);
    for i in 0..20 {
        response.push(password_hash[i] ^ final_hash[i]);
    }

    Ok(response)
}

fn compute_caching_sha2_password(auth_data: &[u8], password: &str) -> Result<Vec<u8>> {
    if password.is_empty() {
        return Ok(vec![0u8]);
    }

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize_reset();

    let mut hasher2 = Sha256::new();
    hasher2.update(&password_hash);
    hasher2.update(auth_data);
    let final_hash = hasher2.finalize_reset();

    let mut response = Vec::with_capacity(32);
    for i in 0..32 {
        response.push(password_hash[i] ^ final_hash[i]);
    }

    Ok(response)
}
