/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use serde::Deserialize;
use small_http::{Request, Status};

use crate::context::Context;
use crate::database::{
    active_symbols, mark_history_synced, pending_history_symbols, replace_ticker_universe,
    update_sync_status,
};
use crate::market_data::fetch_chart;
use crate::models::TickerRecord;

const NASDAQ_LISTED_URL: &str = "https://www.nasdaqtrader.com/dynamic/SymDir/nasdaqlisted.txt";
const OTHER_LISTED_URL: &str = "https://www.nasdaqtrader.com/dynamic/SymDir/otherlisted.txt";
const NASDAQ_SCREENER_URL: &str =
    "https://api.nasdaq.com/api/screener/stocks?tableonly=true&limit=25&download=true";
const EURONEXT_URL: &str = "https://live.euronext.com/en/product_directory/data/stocks-all-places/download?mics=ALXB%2CALXL%2CALXP%2CBGEM%2CENXB%2CENXL%2CETLX%2CEXGM%2CMERK%2CMIVX%2CMLXB%2CMTAA%2CMTAH%2CTNLA%2CTNLB%2CXAMC%2CXAMS%2CXATL%2CXBRU%2CXESM%2CXLDN%2CXLIS%2CXMLI%2CXMSM%2CXOAS%2CXOSL%2CXPAR%2CXPMC";

#[derive(Deserialize)]
struct NasdaqScreenerEnvelope {
    data: Option<NasdaqScreenerData>,
}

#[derive(Deserialize)]
struct NasdaqScreenerData {
    #[serde(default)]
    rows: Vec<NasdaqScreenerRow>,
}

#[derive(Deserialize)]
struct NasdaqScreenerRow {
    symbol: String,
    volume: Option<String>,
    #[serde(rename = "marketCap")]
    market_cap: Option<String>,
}

pub(crate) fn start(context: Context) {
    if env::var("SYNC_ENABLED").is_ok_and(|value| value.eq_ignore_ascii_case("false")) {
        _ = update_sync_status(
            &context.database,
            "disabled",
            "idle",
            0,
            0,
            None,
            "Background synchronization disabled by configuration",
            0,
        );
        return;
    }

    thread::Builder::new()
        .name("market-sync".to_string())
        .spawn(move || run_forever(context))
        .expect("Failed to spawn market synchronization thread");
}

fn run_forever(context: Context) {
    loop {
        if let Err(error) = run_once(&context) {
            log::error!("Market synchronization failed: {error:#}");
            _ = update_sync_status(
                &context.database,
                "retry",
                "failed",
                0,
                0,
                None,
                &format!("Sync stopped: {error}"),
                1,
            );
            thread::sleep(Duration::from_secs(5 * 60));
        } else {
            thread::sleep(Duration::from_secs(60 * 60));
        }
    }
}

fn run_once(context: &Context) -> Result<()> {
    update_sync_status(
        &context.database,
        "universe",
        "running",
        0,
        4,
        None,
        "Downloading official Nasdaq symbol directories",
        0,
    )?;
    let nasdaq = fetch_text(NASDAQ_LISTED_URL)?;
    update_sync_status(
        &context.database,
        "universe",
        "running",
        1,
        4,
        None,
        "Downloaded Nasdaq-listed instruments",
        0,
    )?;
    let other = fetch_text(OTHER_LISTED_URL)?;
    let mut tickers = parse_nasdaq_listed(&nasdaq);
    tickers.extend(parse_other_listed(&other));
    update_sync_status(
        &context.database,
        "universe",
        "running",
        2,
        4,
        None,
        "Ranking US instruments by volume and market capitalization",
        0,
    )?;
    match fetch_text(NASDAQ_SCREENER_URL)
        .and_then(|input| apply_nasdaq_popularity(&mut tickers, &input))
    {
        Ok(()) => {}
        Err(error) => log::warn!("Nasdaq popularity ranking was unavailable: {error:#}"),
    }
    update_sync_status(
        &context.database,
        "universe",
        "running",
        3,
        4,
        None,
        "Downloading the official Euronext equity directory",
        0,
    )?;
    let euronext = fetch_text(EURONEXT_URL)?;
    tickers.extend(parse_euronext(&euronext)?);
    if tickers.len() < 1_000 {
        bail!("symbol directory returned too few instruments");
    }
    replace_ticker_universe(&context.database, &tickers)?;

    let symbols = pending_history_symbols(&context.database)?;
    let total = i64::try_from(symbols.len()).unwrap_or(i64::MAX);
    let delay = env::var("SYNC_REQUEST_DELAY_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(750);
    let mut errors = 0_i64;
    update_sync_status(
        &context.database,
        "history",
        "running",
        0,
        total,
        None,
        "Starting resumable maximum daily-history import",
        errors,
    )?;

    for (index, symbol) in symbols.iter().enumerate() {
        let current = i64::try_from(index).unwrap_or(i64::MAX);
        update_sync_status(
            &context.database,
            "history",
            "running",
            current,
            total,
            Some(symbol.clone()),
            &format!("Importing all daily history for {symbol}"),
            errors,
        )?;
        match fetch_chart(context, symbol, "max") {
            Ok(_) => mark_history_synced(&context.database, symbol)?,
            Err(error) => {
                errors += 1;
                log::warn!("Historical import failed for {symbol}: {error:#}");
            }
        }
        thread::sleep(Duration::from_millis(delay));
    }

    update_sync_status(
        &context.database,
        "history",
        "complete",
        total,
        total,
        symbols.last().cloned(),
        if errors == 0 {
            "Historical import is complete"
        } else {
            "Historical import finished; failed symbols will retry"
        },
        errors,
    )?;

    let symbols = active_symbols(&context.database)?;
    let total = i64::try_from(symbols.len()).unwrap_or(i64::MAX);
    update_sync_status(
        &context.database,
        "current",
        "running",
        0,
        total,
        None,
        "Refreshing current five-minute price bars across the local universe",
        errors,
    )?;
    for (index, symbol) in symbols.iter().enumerate() {
        let current = i64::try_from(index).unwrap_or(i64::MAX);
        if let Err(error) = fetch_chart(context, symbol, "1d") {
            errors += 1;
            log::warn!("Current-price refresh failed for {symbol}: {error:#}");
        }
        update_sync_status(
            &context.database,
            "current",
            "running",
            current + 1,
            total,
            Some(symbol.clone()),
            &format!("Stored the latest available prices for {symbol}"),
            errors,
        )?;
        thread::sleep(Duration::from_millis(delay));
    }

    update_sync_status(
        &context.database,
        "current",
        "complete",
        total,
        total,
        symbols.last().cloned(),
        "Current-price sweep is complete; the next sweep starts in one hour",
        errors,
    )?;
    Ok(())
}

fn fetch_text(url: &str) -> Result<String> {
    let response = Request::get(url)
        .header(
            "User-Agent",
            "plaatmoney/0.1 (self-hosted personal finance dashboard)",
        )
        .fetch()
        .context("symbol-directory request failed")?;
    if response.status != Status::Ok {
        bail!("symbol directory returned {}", response.status);
    }
    String::from_utf8(response.body).context("symbol directory was not UTF-8")
}

fn parse_nasdaq_listed(input: &str) -> Vec<TickerRecord> {
    parse_directory(input, |columns| {
        if columns.len() < 7 || columns[3] != "N" {
            return None;
        }
        Some(TickerRecord {
            symbol: us_provider_symbol(columns[0]),
            local_symbol: columns[0].to_string(),
            name: clean_name(columns[1]),
            exchange: "NASDAQ".to_string(),
            mic: "XNAS".to_string(),
            isin: None,
            market_category: columns[2].to_string(),
            instrument_type: if columns[6] == "Y" { "ETF" } else { "EQUITY" }.to_string(),
            source: "NASDAQ_TRADER".to_string(),
            popularity_score: priority_boost(columns[0]),
        })
    })
}

fn parse_other_listed(input: &str) -> Vec<TickerRecord> {
    parse_directory(input, |columns| {
        if columns.len() < 8 || columns[6] != "N" {
            return None;
        }
        Some(TickerRecord {
            symbol: us_provider_symbol(columns[0]),
            local_symbol: columns[0].to_string(),
            name: clean_name(columns[1]),
            exchange: match columns[2] {
                "A" => "NYSE American",
                "N" => "NYSE",
                "P" => "NYSE Arca",
                "Z" => "Cboe BZX",
                "V" => "IEX",
                other => other,
            }
            .to_string(),
            mic: match columns[2] {
                "A" => "XASE",
                "N" => "XNYS",
                "P" => "ARCX",
                "Z" => "BATS",
                "V" => "IEXG",
                _ => "",
            }
            .to_string(),
            isin: None,
            market_category: columns[2].to_string(),
            instrument_type: if columns[4] == "Y" { "ETF" } else { "EQUITY" }.to_string(),
            source: "NASDAQ_TRADER".to_string(),
            popularity_score: priority_boost(columns[0]),
        })
    })
}

fn parse_euronext(input: &str) -> Result<Vec<TickerRecord>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(input.trim_start_matches('\u{feff}').as_bytes());
    let mut tickers = Vec::new();
    for record in reader.records() {
        let columns = record.context("invalid Euronext CSV")?;
        if columns.len() < 4 {
            continue;
        }
        let name = columns[0].trim();
        let isin = columns[1].trim();
        let local_symbol = columns[2].trim();
        let market = columns[3].trim();
        if name.is_empty() || isin.is_empty() || local_symbol.is_empty() || market.is_empty() {
            continue;
        }
        let volume = parse_number(columns.get(11));
        let turnover = parse_number(columns.get(12));
        tickers.push(TickerRecord {
            symbol: euronext_provider_symbol(local_symbol, market),
            local_symbol: local_symbol.to_string(),
            name: name.to_string(),
            exchange: market.to_string(),
            mic: euronext_mic(market).to_string(),
            isin: Some(isin.to_string()),
            market_category: market.to_string(),
            instrument_type: "EQUITY".to_string(),
            source: "EURONEXT".to_string(),
            popularity_score: priority_boost(local_symbol).max(volume + turnover.sqrt() * 100.0),
        });
    }
    Ok(tickers)
}

fn apply_nasdaq_popularity(tickers: &mut [TickerRecord], input: &str) -> Result<()> {
    let envelope: NasdaqScreenerEnvelope =
        serde_json::from_str(input).context("invalid Nasdaq screener JSON")?;
    let rows = envelope
        .data
        .context("Nasdaq screener response contained no data")?
        .rows;
    let scores = rows
        .into_iter()
        .map(|row| {
            let volume = parse_number(row.volume.as_deref());
            let market_cap = parse_number(row.market_cap.as_deref());
            (
                us_provider_symbol(&row.symbol),
                volume + market_cap.sqrt() * 100.0,
            )
        })
        .collect::<HashMap<_, _>>();
    for ticker in tickers {
        if let Some(score) = scores.get(&ticker.symbol) {
            ticker.popularity_score = ticker.popularity_score.max(*score);
        }
    }
    Ok(())
}

fn parse_number(value: Option<&str>) -> f64 {
    value
        .unwrap_or_default()
        .replace([',', '$'], "")
        .parse()
        .unwrap_or_default()
}

fn priority_boost(symbol: &str) -> f64 {
    const FIRST: &[&str] = &[
        "SPY", "QQQ", "AAPL", "MSFT", "NVDA", "AMZN", "GOOGL", "META", "TSLA", "BRK.B", "VTI",
        "VOO", "IWM", "DIA", "JPM", "V", "MA", "LLY", "AVGO", "ASML", "SAP", "MC", "SHEL",
        "NOVO-B", "AIR", "TTE", "OR", "SAN", "INGA", "ADYEN",
    ];
    FIRST
        .iter()
        .position(|candidate| *candidate == symbol)
        .map_or(0.0, |index| 1.0e15 - index as f64 * 1.0e10)
}

fn us_provider_symbol(symbol: &str) -> String {
    symbol.replace('.', "-")
}

fn euronext_provider_symbol(symbol: &str, market: &str) -> String {
    let suffix = if market.contains("Amsterdam") {
        ".AS"
    } else if market.contains("Brussels") {
        ".BR"
    } else if market.contains("Paris") {
        ".PA"
    } else if market.contains("Lisbon") {
        ".LS"
    } else if market.contains("Oslo") || market == "Oslo Børs" {
        ".OL"
    } else if market.contains("Milan")
        || matches!(
            market,
            "EuroTLX" | "Euronext Global Equity Market" | "Trading After Hours"
        )
    {
        ".MI"
    } else if market.contains("Dublin") {
        ".IR"
    } else {
        return format!("{symbol}.EURONEXT");
    };
    format!("{symbol}{suffix}")
}

fn euronext_mic(market: &str) -> &'static str {
    if market.contains("Amsterdam") {
        "XAMS"
    } else if market.contains("Brussels") {
        "XBRU"
    } else if market.contains("Paris") {
        "XPAR"
    } else if market.contains("Lisbon") {
        "XLIS"
    } else if market.contains("Oslo") || market == "Oslo Børs" {
        "XOSL"
    } else if market.contains("Milan") {
        "MTAA"
    } else if market.contains("Dublin") {
        "XMSM"
    } else {
        ""
    }
}

fn parse_directory(
    input: &str,
    parse: impl Fn(&[&str]) -> Option<TickerRecord>,
) -> Vec<TickerRecord> {
    input
        .lines()
        .skip(1)
        .filter(|line| !line.starts_with("File Creation Time"))
        .filter_map(|line| parse(&line.split('|').collect::<Vec<_>>()))
        .collect()
}

fn clean_name(name: &str) -> String {
    name.split(" - ").next().unwrap_or(name).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nasdaq_directory_and_ignores_test_rows() {
        let input = "Symbol|Security Name|Market Category|Test Issue|Financial Status|Round Lot Size|ETF|NextShares\nAAPL|Apple Inc. - Common Stock|Q|N|N|100|N|N\nTEST|Test Issue|Q|Y|N|100|N|N\nFile Creation Time: 0915202617:03|||||||\n";
        let tickers = parse_nasdaq_listed(input);
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].symbol, "AAPL");
        assert_eq!(tickers[0].name, "Apple Inc.");
        assert_eq!(tickers[0].instrument_type, "EQUITY");
    }

    #[test]
    fn parses_other_exchange_etf() {
        let input = "ACT Symbol|Security Name|Exchange|CQS Symbol|ETF|Round Lot Size|Test Issue|NASDAQ Symbol\nAAA|Example ETF|P|AAA|Y|100|N|AAA\n";
        let tickers = parse_other_listed(input);
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].exchange, "NYSE Arca");
        assert_eq!(tickers[0].instrument_type, "ETF");
    }

    #[test]
    fn parses_euronext_directory_and_maps_provider_symbol() {
        let input = "Name;ISIN;Symbol;Market;Currency;Open;High;Low;Last;Time;Zone;Volume;Turnover\n\"European Equities\"\n\"15 Sep 2026\"\n\"All datapoints provided as of end of last active trading day.\"\nASML HOLDING;NL0010273215;ASML;Euronext Amsterdam;EUR;;;;;;;1200000;900000000\n";
        let tickers = parse_euronext(input).unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].symbol, "ASML.AS");
        assert_eq!(tickers[0].local_symbol, "ASML");
        assert_eq!(tickers[0].isin.as_deref(), Some("NL0010273215"));
        assert_eq!(tickers[0].exchange, "Euronext Amsterdam");
        assert!(tickers[0].popularity_score > 1.0e14);
    }

    #[test]
    fn applies_nasdaq_volume_and_market_cap_ranking() {
        let input = "Symbol|Security Name|Market Category|Test Issue|Financial Status|Round Lot Size|ETF|NextShares\nXYZ|Example Inc.|Q|N|N|100|N|N\n";
        let mut tickers = parse_nasdaq_listed(input);
        let screener = r#"{"data":{"rows":[{"symbol":"XYZ","volume":"2500000","marketCap":"10000000000.00"}]}}"#;
        apply_nasdaq_popularity(&mut tickers, screener).unwrap();
        assert_eq!(tickers[0].popularity_score, 12_500_000.0);
    }
}
