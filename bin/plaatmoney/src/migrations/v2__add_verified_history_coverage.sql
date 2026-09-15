ALTER TABLE tickers ADD COLUMN history_start INTEGER;
ALTER TABLE tickers ADD COLUMN history_end INTEGER;
ALTER TABLE tickers ADD COLUMN history_interval TEXT;

-- Earlier imports could be silently downsampled by Yahoo while labelled as daily.
-- Remove those ambiguous rows and rebuild them with bounded, verified windows.
DELETE FROM price_bars WHERE interval = '1d';

UPDATE tickers
SET last_history_sync = NULL,
    history_start = NULL,
    history_end = NULL,
    history_interval = NULL;
