/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bsql::{Connection, FromRow, execute_args, query_args};
use serde::Serialize;

use crate::models::{ChartResponse, SearchResult, TickerRecord};

#[derive(FromRow)]
struct LocalTicker {
    symbol: String,
    name: String,
    exchange: String,
    instrument_type: String,
}

#[derive(Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncStatus {
    pub(crate) phase: String,
    pub(crate) status: String,
    pub(crate) current: i64,
    pub(crate) total: i64,
    pub(crate) cursor: Option<String>,
    pub(crate) message: String,
    pub(crate) error_count: i64,
    pub(crate) updated_at: i64,
    pub(crate) instrument_count: i64,
    pub(crate) history_synced_count: i64,
    pub(crate) stored_bar_count: i64,
}

pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) fn local_search(database: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let pattern = format!("%{}%", query.trim());
    let rows = query_args!(
        LocalTicker,
        database,
        "SELECT symbol, name, exchange, instrument_type
         FROM tickers
         WHERE is_active = 1 AND (symbol LIKE :symbol OR local_symbol LIKE :local_symbol OR name LIKE :name OR isin LIKE :isin)
         ORDER BY CASE WHEN symbol = :exact OR local_symbol = :exact THEN 0
              WHEN symbol LIKE :prefix OR local_symbol LIKE :prefix THEN 1 ELSE 2 END, symbol
         LIMIT 8",
        Args {
            symbol: pattern.clone(),
            local_symbol: pattern.clone(),
            name: pattern.clone(),
            isin: pattern,
            exact: query.trim().to_ascii_uppercase(),
            prefix: format!("{}%", query.trim()),
        }
    )?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .map(|ticker| SearchResult {
            symbol: ticker.symbol,
            name: ticker.name,
            exchange: ticker.exchange,
            instrument_type: ticker.instrument_type,
            sector: None,
            industry: None,
        })
        .collect())
}

pub(crate) fn replace_ticker_universe(
    database: &Connection,
    tickers: &[TickerRecord],
) -> Result<()> {
    database.transaction(|transaction| -> Result<()> {
        transaction.execute(
            "UPDATE tickers SET is_active = 0 WHERE source IN ('NASDAQ_TRADER', 'EURONEXT')",
            (),
        )?;
        for ticker in tickers {
            execute_args!(
                transaction,
                "INSERT INTO tickers(symbol, local_symbol, name, exchange, mic, isin, market_category, instrument_type, source, popularity_score, is_active, updated_at)
                 VALUES(:symbol, :local_symbol, :name, :exchange, :mic, :isin, :market_category, :instrument_type, :source, :popularity_score, 1, :updated_at)
                 ON CONFLICT(symbol) DO UPDATE SET local_symbol = excluded.local_symbol,
                 name = excluded.name, exchange = excluded.exchange, mic = excluded.mic, isin = excluded.isin,
                 market_category = excluded.market_category, instrument_type = excluded.instrument_type,
                 source = excluded.source, popularity_score = excluded.popularity_score,
                 is_active = 1, updated_at = excluded.updated_at",
                Args {
                    symbol: ticker.symbol.clone(),
                    local_symbol: ticker.local_symbol.clone(),
                    name: ticker.name.clone(),
                    exchange: ticker.exchange.clone(),
                    mic: ticker.mic.clone(),
                    isin: ticker.isin.clone(),
                    market_category: ticker.market_category.clone(),
                    instrument_type: ticker.instrument_type.clone(),
                    source: ticker.source.clone(),
                    popularity_score: ticker.popularity_score,
                    updated_at: now(),
                }
            )?;
        }
        Ok(())
    })
}

pub(crate) fn save_chart(database: &Connection, chart: &ChartResponse) -> Result<()> {
    database.transaction(|transaction| -> Result<()> {
        execute_args!(
            transaction,
            "INSERT INTO tickers(symbol, local_symbol, name, exchange, mic, isin, market_category, instrument_type, source, popularity_score, is_active, updated_at)
             VALUES(:symbol, :symbol, :name, :exchange, '', NULL, '', :instrument_type, 'YAHOO_FINANCE', 0, 1, :updated_at)
             ON CONFLICT(symbol) DO UPDATE SET name = excluded.name, exchange = excluded.exchange,
             instrument_type = excluded.instrument_type, updated_at = excluded.updated_at",
            Args {
                symbol: chart.symbol.clone(),
                name: chart.name.clone(),
                exchange: chart.exchange.clone(),
                instrument_type: chart.instrument_type.clone(),
                updated_at: now(),
            }
        )?;
        for point in &chart.points {
            execute_args!(
                transaction,
                "INSERT INTO price_bars(symbol, interval, timestamp, open, high, low, close, volume, provider)
                 VALUES(:symbol, :interval, :timestamp, :open, :high, :low, :close, :volume, :provider)
                 ON CONFLICT(symbol, interval, timestamp) DO UPDATE SET open = excluded.open,
                 high = excluded.high, low = excluded.low, close = excluded.close,
                 volume = excluded.volume, provider = excluded.provider",
                Args {
                    symbol: chart.symbol.clone(),
                    interval: chart.interval.clone(),
                    timestamp: point.time,
                    open: point.open,
                    high: point.high,
                    low: point.low,
                    close: point.close,
                    volume: i64::try_from(point.volume).unwrap_or(i64::MAX),
                    provider: chart.provider.to_string(),
                }
            )?;
        }
        Ok(())
    })
}

pub(crate) fn pending_history_symbols(database: &Connection) -> Result<Vec<String>> {
    Ok(database
        .query::<String>(
            "SELECT symbol FROM tickers WHERE is_active = 1 AND last_history_sync IS NULL
             ORDER BY popularity_score DESC, symbol",
            (),
        )?
        .collect::<Result<Vec<_>, _>>()?)
}

pub(crate) fn active_symbols(database: &Connection) -> Result<Vec<String>> {
    Ok(database
        .query::<String>(
            "SELECT symbol FROM tickers WHERE is_active = 1
             ORDER BY popularity_score DESC, symbol",
            (),
        )?
        .collect::<Result<Vec<_>, _>>()?)
}

pub(crate) fn history_cursor(database: &Connection, symbol: &str) -> Result<Option<i64>> {
    Ok(database.query_some::<Option<i64>>(
        "SELECT history_end FROM tickers WHERE symbol = ?",
        (symbol.to_string(),),
    )?)
}

pub(crate) fn update_history_coverage(
    database: &Connection,
    symbol: &str,
    history_start: i64,
    history_end: i64,
) -> Result<()> {
    database.execute(
        "UPDATE tickers
         SET history_start = COALESCE(history_start, ?), history_end = ?, history_interval = '1d'
         WHERE symbol = ?",
        (history_start, history_end, symbol.to_string()),
    )?;
    Ok(())
}

pub(crate) fn mark_history_synced(
    database: &Connection,
    symbol: &str,
    required_end: i64,
) -> Result<()> {
    database.execute(
        "UPDATE tickers SET last_history_sync = ?
         WHERE symbol = ? AND history_interval = '1d' AND history_end >= ?",
        (now(), symbol.to_string(), required_end),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_sync_status(
    database: &Connection,
    phase: &str,
    status: &str,
    current: i64,
    total: i64,
    cursor: Option<String>,
    message: &str,
    error_count: i64,
) -> Result<()> {
    execute_args!(
        database,
        "UPDATE sync_jobs SET phase = :phase, status = :status, current = :current,
         total = :total, cursor = :cursor, message = :message, error_count = :error_count,
         updated_at = :updated_at WHERE name = 'market-bootstrap'",
        Args {
            phase: phase.to_string(),
            status: status.to_string(),
            current: current,
            total: total,
            cursor: cursor,
            message: message.to_string(),
            error_count: error_count,
            updated_at: now(),
        }
    )?;
    Ok(())
}

pub(crate) fn sync_status(database: &Connection) -> Result<SyncStatus> {
    let (phase, status, current, total, cursor, message, error_count, updated_at) = database
        .query_some::<(String, String, i64, i64, Option<String>, String, i64, i64)>(
            "SELECT phase, status, current, total, cursor, message, error_count, updated_at
             FROM sync_jobs WHERE name = 'market-bootstrap'",
            (),
        )?;
    Ok(SyncStatus {
        phase,
        status,
        current,
        total,
        cursor,
        message,
        error_count,
        updated_at,
        instrument_count: database
            .query_some("SELECT COUNT(*) FROM tickers WHERE is_active = 1", ())?,
        history_synced_count: database.query_some(
            "SELECT COUNT(*) FROM tickers WHERE is_active = 1 AND last_history_sync IS NOT NULL",
            (),
        )?,
        stored_bar_count: database.query_some("SELECT COUNT(*) FROM price_bars", ())?,
    })
}
