/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::context::Context;

fn inflate_gzip(data: &[u8]) -> Option<Vec<u8>> {
    // Validate gzip magic and compression method
    if data.len() < 18 || data[0] != 0x1f || data[1] != 0x8b || data[2] != 0x08 {
        return None;
    }
    let flags = data[3];
    let mut offset = 10usize;
    // FEXTRA
    if flags & 0x04 != 0 {
        if offset + 2 > data.len() {
            return None;
        }
        let xlen = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2 + xlen;
    }
    // FNAME - null-terminated string
    if flags & 0x08 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1;
    }
    // FCOMMENT - null-terminated string
    if flags & 0x10 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1;
    }
    // FHCRC
    if flags & 0x02 != 0 {
        offset += 2;
    }
    if offset > data.len().saturating_sub(8) {
        return None;
    }
    // Decompress raw DEFLATE payload (strip 8-byte gzip footer)
    miniz_oxide::inflate::decompress_to_vec(&data[offset..data.len() - 8]).ok()
}

fn try_download_mmdb(mmdb_path: &str) {
    let date = chrono::Utc::now().naive_utc().date();
    let date_str = format!("{date}");
    let year: u32 = date_str[..4].parse().unwrap_or(2025);
    let month: u32 = date_str[5..7].parse().unwrap_or(1);
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    for (y, m) in [(year, month), (prev_year, prev_month)] {
        let url = format!("https://download.db-ip.com/free/dbip-city-lite-{y}-{m:02}.mmdb.gz");
        log::info!("Downloading DB-IP city lite database from {url}...");
        match small_http::Request::get(&url).fetch() {
            Ok(res) if res.status == small_http::Status::Ok => match inflate_gzip(&res.body) {
                Some(decompressed) => match std::fs::write(mmdb_path, &decompressed) {
                    Ok(()) => {
                        log::info!("DB-IP city lite database saved to {mmdb_path}");
                        return;
                    }
                    Err(e) => log::warn!("Failed to write DB-IP database to {mmdb_path}: {e}"),
                },
                None => log::warn!("Failed to decompress DB-IP database"),
            },
            Ok(_) => log::warn!("Unexpected HTTP response when downloading DB-IP database"),
            Err(_) => log::warn!("Failed to connect to download.db-ip.com"),
        }
    }
    log::warn!("Could not download DB-IP city lite database, will fall back to ipinfo.io");
}

pub(crate) fn run(mmdb_path: String, ctx: Context) {
    let is_nonempty = std::fs::metadata(&mmdb_path)
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if !is_nonempty {
        try_download_mmdb(&mmdb_path);
    }
    let is_nonempty = std::fs::metadata(&mmdb_path)
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if is_nonempty {
        match maxminddb::Reader::open_readfile(&mmdb_path) {
            Ok(reader) => {
                log::info!("Using DB-IP city lite database at {mmdb_path}");
                let _ = ctx.maxminddb_reader.set(reader);
            }
            Err(e) => log::warn!("Failed to open DB-IP database at {mmdb_path}: {e}"),
        }
    }
}
