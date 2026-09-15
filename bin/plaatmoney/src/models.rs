/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub(crate) struct TickerRecord {
    pub(crate) symbol: String,
    pub(crate) local_symbol: String,
    pub(crate) name: String,
    pub(crate) exchange: String,
    pub(crate) mic: String,
    pub(crate) isin: Option<String>,
    pub(crate) market_category: String,
    pub(crate) instrument_type: String,
    pub(crate) source: String,
    pub(crate) popularity_score: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartResponse {
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) exchange: String,
    pub(crate) instrument_type: String,
    pub(crate) currency: String,
    pub(crate) timezone: String,
    pub(crate) regular_market_price: f64,
    pub(crate) previous_close: f64,
    pub(crate) change: f64,
    pub(crate) change_percent: f64,
    pub(crate) market_time: i64,
    pub(crate) day_high: Option<f64>,
    pub(crate) day_low: Option<f64>,
    pub(crate) volume: Option<u64>,
    pub(crate) fifty_two_week_high: Option<f64>,
    pub(crate) fifty_two_week_low: Option<f64>,
    pub(crate) range: String,
    pub(crate) interval: String,
    pub(crate) provider: &'static str,
    pub(crate) points: Vec<ChartPoint>,
}

#[derive(Clone, Serialize)]
pub(crate) struct ChartPoint {
    pub(crate) time: i64,
    pub(crate) open: f64,
    pub(crate) high: f64,
    pub(crate) low: f64,
    pub(crate) close: f64,
    pub(crate) volume: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResult {
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) exchange: String,
    pub(crate) instrument_type: String,
    pub(crate) sector: Option<String>,
    pub(crate) industry: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct YahooChartEnvelope {
    pub(crate) chart: YahooChartContainer,
}

#[derive(Deserialize)]
pub(crate) struct YahooChartContainer {
    pub(crate) result: Option<Vec<YahooChart>>,
}

#[derive(Deserialize)]
pub(crate) struct YahooChart {
    pub(crate) meta: YahooMeta,
    pub(crate) timestamp: Option<Vec<i64>>,
    pub(crate) indicators: YahooIndicators,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YahooMeta {
    pub(crate) currency: Option<String>,
    pub(crate) symbol: String,
    pub(crate) first_trade_date: Option<i64>,
    pub(crate) data_granularity: Option<String>,
    pub(crate) full_exchange_name: Option<String>,
    pub(crate) exchange_name: Option<String>,
    pub(crate) instrument_type: Option<String>,
    pub(crate) exchange_timezone_name: Option<String>,
    pub(crate) regular_market_price: Option<f64>,
    pub(crate) regular_market_time: Option<i64>,
    pub(crate) regular_market_day_high: Option<f64>,
    pub(crate) regular_market_day_low: Option<f64>,
    pub(crate) regular_market_volume: Option<u64>,
    pub(crate) fifty_two_week_high: Option<f64>,
    pub(crate) fifty_two_week_low: Option<f64>,
    pub(crate) previous_close: Option<f64>,
    pub(crate) chart_previous_close: Option<f64>,
    pub(crate) long_name: Option<String>,
    pub(crate) short_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct YahooIndicators {
    pub(crate) quote: Vec<YahooQuoteSeries>,
}

#[derive(Deserialize)]
pub(crate) struct YahooQuoteSeries {
    pub(crate) open: Vec<Option<f64>>,
    pub(crate) high: Vec<Option<f64>>,
    pub(crate) low: Vec<Option<f64>>,
    pub(crate) close: Vec<Option<f64>>,
    pub(crate) volume: Vec<Option<u64>>,
}

#[derive(Deserialize)]
pub(crate) struct YahooSearchEnvelope {
    #[serde(default)]
    pub(crate) quotes: Vec<YahooSearchQuote>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YahooSearchQuote {
    pub(crate) symbol: Option<String>,
    pub(crate) longname: Option<String>,
    pub(crate) shortname: Option<String>,
    pub(crate) exchange: Option<String>,
    pub(crate) exch_disp: Option<String>,
    pub(crate) quote_type: Option<String>,
    pub(crate) sector: Option<String>,
    pub(crate) industry: Option<String>,
}
