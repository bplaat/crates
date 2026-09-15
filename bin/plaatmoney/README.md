# PlaatMoney

A self-hosted, read-only stock market dashboard. Search global Yahoo Finance
symbols, inspect current pricing and company basics, and explore intraday through
maximum available price history in a responsive Preact interface. On startup it
imports the official Nasdaq and Euronext symbol directories and incrementally
mirrors daily history into a local SQLite database using `bsql`.

## Run locally

From the workspace root:

```sh
cargo run -p plaatmoney
```

Then open <http://localhost:8080>. Set `PORT` to use a different port.
Data is stored in `DATA_PATH/plaatmoney.db`; `DATA_PATH` defaults to `./data`.

## Docker

```sh
docker build --tag plaatmoney --file bin/plaatmoney/Dockerfile .
docker run --rm -p 8080:8080 -v plaatmoney-data:/data plaatmoney
```

## Data source

The initial adapter proxies Yahoo Finance search and chart endpoints and caches
responses for 15 seconds. Yahoo is an unofficial provider integration and may be
delayed or change without notice. It is suitable for a personal dashboard, not as
a licensed real-time exchange feed or for automated trading decisions.

The background worker downloads keyless official symbol files from Nasdaq Trader
and the Euronext equities directory, then fetches maximum daily history one ticker
at a time. It preserves Euronext ticker, ISIN, and MIC metadata while storing a
Yahoo-compatible provider symbol for price retrieval. Successful tickers are
checkpointed, so restarting the container resumes instead of starting over. Once
history is current, the worker cycles through the universe to store the latest
available five-minute bars. Set `SYNC_ENABLED=false` to disable the worker or
`SYNC_REQUEST_DELAY_MS` to tune the default 750 ms delay between requests.

The import queue is liquidity-first rather than alphabetical. A small set of
widely followed stocks and index ETFs is seeded first, then Nasdaq instruments
are ranked using the screener's volume and market capitalization while Euronext
instruments use the volume and turnover fields in its bulk directory.

## Scope

This first release intentionally contains no accounts, portfolio management,
signals, recommendations, order entry, or broker integration.
