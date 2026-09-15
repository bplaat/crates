CREATE TABLE tickers(
    symbol TEXT PRIMARY KEY,
    local_symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    exchange TEXT NOT NULL,
    mic TEXT NOT NULL,
    isin TEXT,
    market_category TEXT NOT NULL,
    instrument_type TEXT NOT NULL,
    source TEXT NOT NULL,
    popularity_score REAL NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL,
    last_history_sync INTEGER,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX tickers_name_index ON tickers(name);
CREATE INDEX tickers_local_symbol_index ON tickers(local_symbol);
CREATE INDEX tickers_isin_index ON tickers(isin);
CREATE INDEX tickers_history_sync_index ON tickers(is_active, last_history_sync);
CREATE INDEX tickers_popularity_index ON tickers(is_active, popularity_score DESC);

CREATE TABLE price_bars(
    symbol TEXT NOT NULL,
    interval TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume INTEGER NOT NULL,
    provider TEXT NOT NULL,
    PRIMARY KEY(symbol, interval, timestamp),
    FOREIGN KEY(symbol) REFERENCES tickers(symbol) ON DELETE CASCADE
) STRICT;

CREATE INDEX price_bars_lookup_index ON price_bars(symbol, interval, timestamp);

CREATE TABLE sync_jobs(
    name TEXT PRIMARY KEY,
    phase TEXT NOT NULL,
    status TEXT NOT NULL,
    current INTEGER NOT NULL,
    total INTEGER NOT NULL,
    cursor TEXT,
    message TEXT NOT NULL,
    error_count INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

INSERT INTO sync_jobs(name, phase, status, current, total, cursor, message, error_count, updated_at)
VALUES('market-bootstrap', 'startup', 'idle', 0, 0, NULL, 'Waiting to start', 0, 0);
