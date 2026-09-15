/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use small_http::{Request, Status};

use crate::context::{CacheEntry, Context};
use crate::models::*;

const CACHE_TTL: Duration = Duration::from_secs(15);
const DAY_SECONDS: i64 = 24 * 60 * 60;
const HISTORY_WINDOW_SECONDS: i64 = 5 * 365 * DAY_SECONDS;

pub(crate) fn fetch_chart(context: &Context, symbol: &str, range: &str) -> Result<ChartResponse> {
    let interval = interval_for_range(range).context("unsupported chart range")?;
    let query = serde_urlencoded::to_string([
        ("range", range),
        ("interval", interval),
        ("includePrePost", "true"),
        ("events", "div,splits"),
    ])?;
    let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?{query}");

    let chart = fetch_yahoo_chart(context, &url)?;
    let chart = normalize_chart(chart, range, interval)?;
    if let Err(error) = crate::database::save_chart(&context.database, &chart) {
        log::warn!("Failed to store {} price history: {error}", chart.symbol);
    }
    Ok(chart)
}

pub(crate) fn fetch_complete_daily_history(
    context: &Context,
    symbol: &str,
    request_delay: Duration,
) -> Result<()> {
    let metadata_url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?range=1d&interval=1d&includePrePost=false&events=div%2Csplits"
    );
    let metadata = fetch_yahoo_chart(context, &metadata_url)?;
    let first_trade_date = metadata
        .meta
        .first_trade_date
        .context("Yahoo did not provide the first trade date")?;
    let required_end = crate::database::now() + DAY_SECONDS;
    let mut window_start = crate::database::history_cursor(&context.database, symbol)?
        .unwrap_or(first_trade_date)
        .max(first_trade_date);

    while window_start < required_end {
        let window_end = window_start
            .saturating_add(HISTORY_WINDOW_SECONDS)
            .min(required_end);
        let query = serde_urlencoded::to_string([
            ("period1", window_start.to_string()),
            ("period2", window_end.to_string()),
            ("interval", "1d".to_string()),
            ("includePrePost", "false".to_string()),
            ("events", "div,splits".to_string()),
        ])?;
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?{query}");
        let yahoo_chart = fetch_yahoo_chart(context, &url)?;
        if yahoo_chart.meta.data_granularity.as_deref() != Some("1d") {
            bail!(
                "Yahoo returned {:?} granularity for a verified daily window",
                yahoo_chart.meta.data_granularity
            );
        }
        let chart = normalize_chart(yahoo_chart, "max", "1d")?;
        crate::database::save_chart(&context.database, &chart)?;
        crate::database::update_history_coverage(
            &context.database,
            symbol,
            first_trade_date,
            window_end,
        )?;
        window_start = window_end;
        if window_start < required_end {
            std::thread::sleep(request_delay);
        }
    }

    crate::database::mark_history_synced(&context.database, symbol, required_end)
}

pub(crate) fn fetch_search(context: &Context, query: &str) -> Result<Vec<SearchResult>> {
    let query =
        serde_urlencoded::to_string([("q", query), ("quotesCount", "8"), ("newsCount", "0")])?;
    let url = format!("https://query1.finance.yahoo.com/v1/finance/search?{query}");
    let body = fetch_cached(context, &url)?;
    let envelope: YahooSearchEnvelope =
        serde_json::from_slice(&body).context("invalid search response")?;
    Ok(envelope
        .quotes
        .into_iter()
        .filter_map(|quote| {
            let symbol = quote.symbol?;
            let name = quote.longname.or(quote.shortname)?;
            Some(SearchResult {
                symbol,
                name,
                exchange: quote.exch_disp.or(quote.exchange).unwrap_or_default(),
                instrument_type: quote.quote_type.unwrap_or_else(|| "UNKNOWN".to_string()),
                sector: quote.sector,
                industry: quote.industry,
            })
        })
        .collect())
}

fn fetch_cached(context: &Context, url: &str) -> Result<Vec<u8>> {
    if let Some(entry) = context
        .cache
        .lock()
        .expect("market data cache lock poisoned")
        .get(url)
        .filter(|entry| entry.stored_at.elapsed() < CACHE_TTL)
    {
        return Ok(entry.body.clone());
    }

    let response = context
        .client
        .lock()
        .expect("market data client lock poisoned")
        .fetch(Request::get(url))
        .context("market data provider request failed")?;
    if response.status != Status::Ok {
        bail!("market data provider returned {}", response.status);
    }
    let body = response.body;
    context
        .cache
        .lock()
        .expect("market data cache lock poisoned")
        .insert(
            url.to_string(),
            CacheEntry {
                stored_at: std::time::Instant::now(),
                body: body.clone(),
            },
        );
    Ok(body)
}

fn fetch_yahoo_chart(context: &Context, url: &str) -> Result<YahooChart> {
    let body = fetch_cached(context, url)?;
    let envelope: YahooChartEnvelope =
        serde_json::from_slice(&body).context("invalid chart response")?;
    envelope
        .chart
        .result
        .and_then(|mut result| result.pop())
        .context("ticker was not found")
}

fn normalize_chart(chart: YahooChart, range: &str, interval: &str) -> Result<ChartResponse> {
    let series = chart
        .indicators
        .quote
        .first()
        .context("chart contains no quote series")?;
    let timestamps = chart.timestamp.unwrap_or_default();
    let mut points = Vec::with_capacity(timestamps.len());
    for (index, time) in timestamps.into_iter().enumerate() {
        let Some((open, high, low, close)) = series
            .open
            .get(index)
            .copied()
            .flatten()
            .zip(series.high.get(index).copied().flatten())
            .zip(series.low.get(index).copied().flatten())
            .zip(series.close.get(index).copied().flatten())
            .map(|(((open, high), low), close)| (open, high, low, close))
        else {
            continue;
        };
        points.push(ChartPoint {
            time,
            open,
            high,
            low,
            close,
            volume: series.volume.get(index).copied().flatten().unwrap_or(0),
        });
    }

    let meta = chart.meta;
    let actual_interval = meta
        .data_granularity
        .clone()
        .unwrap_or_else(|| interval.to_string());
    let price = meta
        .regular_market_price
        .or_else(|| points.last().map(|point| point.close))
        .context("chart contains no current price")?;
    let previous_close = meta
        .previous_close
        .or(meta.chart_previous_close)
        .unwrap_or(price);
    let change = price - previous_close;
    let change_percent = if previous_close == 0.0 {
        0.0
    } else {
        change / previous_close * 100.0
    };

    Ok(ChartResponse {
        name: meta
            .long_name
            .or(meta.short_name)
            .unwrap_or_else(|| meta.symbol.clone()),
        symbol: meta.symbol,
        exchange: meta
            .full_exchange_name
            .or(meta.exchange_name)
            .unwrap_or_default(),
        instrument_type: meta.instrument_type.unwrap_or_default(),
        currency: meta.currency.unwrap_or_default(),
        timezone: meta
            .exchange_timezone_name
            .unwrap_or_else(|| "UTC".to_string()),
        regular_market_price: price,
        previous_close,
        change,
        change_percent,
        market_time: meta.regular_market_time.unwrap_or_default(),
        day_high: meta.regular_market_day_high,
        day_low: meta.regular_market_day_low,
        volume: meta.regular_market_volume,
        fifty_two_week_high: meta.fifty_two_week_high,
        fifty_two_week_low: meta.fifty_two_week_low,
        range: range.to_string(),
        interval: actual_interval,
        provider: "Yahoo Finance",
        points,
    })
}

pub(crate) fn valid_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 20
        && symbol
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'^' | b'='))
}

fn interval_for_range(range: &str) -> Option<&'static str> {
    match range {
        "1d" => Some("5m"),
        "5d" => Some("15m"),
        "1mo" => Some("1h"),
        "6mo" | "1y" => Some("1d"),
        "5y" => Some("1wk"),
        "max" => Some("1d"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_supported_symbols() {
        assert!(valid_symbol("ASML.AS"));
        assert!(valid_symbol("BRK-B"));
        assert!(valid_symbol("^GSPC"));
        assert!(!valid_symbol("../../etc/passwd"));
        assert!(!valid_symbol("AAPL?range=max"));
    }

    #[test]
    fn chooses_bounded_intervals() {
        assert_eq!(interval_for_range("1d"), Some("5m"));
        assert_eq!(interval_for_range("5y"), Some("1wk"));
        assert_eq!(interval_for_range("max"), Some("1d"));
    }
}
