/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;
use small_http::{Request, Response, Status};
use small_websocket::Message;

use crate::Context;
use crate::market_data::{fetch_chart, fetch_search, valid_symbol};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: &'a str,
}

pub(crate) fn health(_: &Request, _: &Context) -> Result<Response> {
    Ok(Response::with_json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }))
}

pub(crate) fn search(request: &Request, context: &Context) -> Result<Response> {
    let query = parse_query(request).remove("q").unwrap_or_default();
    if query.trim().is_empty() || query.len() > 80 {
        return Ok(
            Response::with_status(Status::BadRequest).json(ErrorResponse {
                error: "query must contain between 1 and 80 characters",
            }),
        );
    }
    let local_results = crate::database::local_search(&context.database, query.trim())?;
    if !local_results.is_empty() {
        return Ok(Response::with_json(local_results));
    }
    match fetch_search(context, query.trim()) {
        Ok(results) => Ok(Response::with_json(results)),
        Err(error) => {
            log::warn!("Ticker search failed: {error:#}");
            Ok(
                Response::with_status(Status::BadGateway).json(ErrorResponse {
                    error: "market data provider is unavailable",
                }),
            )
        }
    }
}

pub(crate) fn sync_status(_: &Request, context: &Context) -> Result<Response> {
    Ok(Response::with_json(crate::database::sync_status(
        &context.database,
    )?))
}

pub(crate) fn sync_stream(request: &Request, context: &Context) -> Result<Response> {
    let context = context.clone();
    Ok(small_websocket::upgrade(request, move |mut socket| {
        _ = socket.set_write_timeout(Some(std::time::Duration::from_secs(5)));
        loop {
            let status = match crate::database::sync_status(&context.database) {
                Ok(status) => status,
                Err(error) => {
                    log::warn!("Failed to read sync status: {error}");
                    break;
                }
            };
            let Ok(payload) = serde_json::to_string(&status) else {
                break;
            };
            if socket.send(Message::Text(payload)).is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }))
}

pub(crate) fn chart(request: &Request, context: &Context) -> Result<Response> {
    let encoded_symbol = request
        .params
        .get("symbol")
        .map(String::as_str)
        .unwrap_or("");
    let symbol = decode_path_segment(encoded_symbol);
    let range = parse_query(request)
        .remove("range")
        .unwrap_or_else(|| "1d".to_string());
    if !valid_symbol(&symbol) {
        return Ok(
            Response::with_status(Status::BadRequest).json(ErrorResponse {
                error: "invalid ticker symbol",
            }),
        );
    }
    match fetch_chart(context, &symbol, &range) {
        Ok(chart) => Ok(Response::with_json(chart)),
        Err(error) => {
            log::warn!("Chart request for {symbol} failed: {error:#}");
            Ok(
                Response::with_status(Status::BadGateway).json(ErrorResponse {
                    error: "ticker was not found or the market data provider is unavailable",
                }),
            )
        }
    }
}

fn decode_path_segment(value: &str) -> String {
    serde_urlencoded::from_str::<HashMap<String, String>>(&format!("value={value}"))
        .ok()
        .and_then(|mut values| values.remove("value"))
        .unwrap_or_else(|| value.to_string())
}

fn parse_query(request: &Request) -> HashMap<String, String> {
    request
        .url
        .query()
        .and_then(|query| serde_urlencoded::from_str(query).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_index_symbols_from_paths() {
        assert_eq!(decode_path_segment("%5EGSPC"), "^GSPC");
        assert_eq!(decode_path_segment("ASML.AS"), "ASML.AS");
    }
}
