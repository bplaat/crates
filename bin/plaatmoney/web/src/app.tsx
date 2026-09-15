/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import type { Chart } from './api.ts';
import { getChart } from './api.ts';
import { MarketTile } from './components/market-tile.tsx';
import { PriceChart } from './components/price-chart.tsx';
import { SearchBox } from './components/search-box.tsx';
import { SyncMonitor } from './components/sync-monitor.tsx';
import { formatCompact, formatDateTime, formatPrice } from './format.ts';

const RANGES = [
    ['1d', '1D'],
    ['5d', '5D'],
    ['1mo', '1M'],
    ['6mo', '6M'],
    ['1y', '1Y'],
    ['5y', '5Y'],
    ['max', 'MAX'],
] as const;

const WATCHLIST = ['AAPL', 'MSFT', 'NVDA', 'ASML.AS', 'SAP.DE', 'MC.PA'];

export function App() {
    const [symbol, setSymbol] = useState('AAPL');
    const [range, setRange] = useState('1d');
    const [chart, setChart] = useState<Chart | null>(null);
    const [marketTiles, setMarketTiles] = useState<Chart[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const controller = new AbortController();
        setLoading(true);
        setError(null);
        const load = () => {
            getChart(symbol, range, controller.signal)
                .then(setChart)
                .catch((nextError: unknown) => {
                    if (nextError instanceof DOMException && nextError.name === 'AbortError') return;
                    setError(nextError instanceof Error ? nextError.message : 'Quote data is unavailable');
                })
                .finally(() => setLoading(false));
        };
        load();
        const refresh = window.setInterval(load, 15_000);
        return () => {
            controller.abort();
            window.clearInterval(refresh);
        };
    }, [symbol, range]);

    useEffect(() => {
        const controller = new AbortController();
        Promise.all(WATCHLIST.map((item) => getChart(item, '1d', controller.signal).catch(() => null))).then((items) => {
            setMarketTiles(items.filter((item): item is Chart => item !== null));
        });
        return () => controller.abort();
    }, []);

    function selectSymbol(nextSymbol: string) {
        setSymbol(nextSymbol);
        setRange('1d');
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    const positive = (chart?.change ?? 0) >= 0;

    return (
        <div class="app-shell">
            <a class="skip-link" href="#main">Skip to market data</a>
            <header class="topbar">
                <a class="brand" href="/" aria-label="PlaatMoney home">
                    <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i><i></i></span>
                    <span><strong>Plaat</strong>Money</span>
                </a>
                <SearchBox onSelect={selectSymbol} />
                <div class="feed-status">
                    <span class="pulse-dot" aria-hidden="true"></span>
                    <span>Auto-refresh 15s</span>
                </div>
            </header>

            <div class="ticker-tape" aria-label="Popular market instruments">
                <span class="tape-label">Market pulse</span>
                <div class="tape-items">
                    {marketTiles.map((item) => (
                        <button type="button" key={item.symbol} onClick={() => selectSymbol(item.symbol)}>
                            <strong>{item.symbol}</strong>
                            <span>{item.regularMarketPrice.toFixed(2)}</span>
                            <em class={item.change >= 0 ? 'is-positive' : 'is-negative'}>
                                {item.change >= 0 ? '+' : ''}{item.changePercent.toFixed(2)}%
                            </em>
                        </button>
                    ))}
                    {marketTiles.length === 0 && <span class="tape-loading">Loading global markets...</span>}
                </div>
            </div>

            <div class="workspace">
                <aside class="sidebar">
                    <div class="side-section">
                        <p class="side-label">Watchlist</p>
                        <nav aria-label="Watchlist">
                            {WATCHLIST.map((item) => {
                                const quote = marketTiles.find((candidate) => candidate.symbol === item);
                                return (
                                    <button
                                        type="button"
                                        key={item}
                                        class={symbol === item ? 'active' : ''}
                                        onClick={() => selectSymbol(item)}
                                    >
                                        <span><strong>{item}</strong><small>{quote?.name ?? 'Loading...'}</small></span>
                                        {quote && <em class={quote.change >= 0 ? 'is-positive' : 'is-negative'}>{quote.changePercent.toFixed(2)}%</em>}
                                    </button>
                                );
                            })}
                        </nav>
                    </div>
                    <div class="source-note">
                        <span>DATA SOURCE</span>
                        <strong>Yahoo Finance</strong>
                        <p>Unofficial integration. Quotes may be delayed. Not for order execution.</p>
                    </div>
                    <p class="version">Self-hosted v{__APP_VERSION__}</p>
                </aside>

                <main id="main" class="main-content">
                    {error ? (
                        <section class="error-state" role="alert">
                            <span>DATA INTERRUPT</span>
                            <h1>{symbol} could not be loaded</h1>
                            <p>{error}. Check the symbol or try again in a moment.</p>
                            <button type="button" onClick={() => selectSymbol('AAPL')}>Return to AAPL</button>
                        </section>
                    ) : chart ? (
                        <>
                            <section class={`quote-hero ${loading ? 'is-refreshing' : ''}`}>
                                <div class="quote-identity">
                                    <div class="eyebrow"><span>{chart.exchange}</span><span>{chart.instrumentType}</span><span>{chart.currency}</span></div>
                                    <h1>{chart.name}</h1>
                                    <p>{chart.symbol} / {chart.timezone}</p>
                                </div>
                                <div class="quote-price">
                                    <strong>{formatPrice(chart.regularMarketPrice, chart.currency)}</strong>
                                    <div class={positive ? 'quote-change is-positive' : 'quote-change is-negative'}>
                                        <span>{positive ? '+' : ''}{chart.change.toFixed(2)}</span>
                                        <span>{positive ? '+' : ''}{chart.changePercent.toFixed(2)}%</span>
                                    </div>
                                    <p>As of {formatDateTime(chart.marketTime)}</p>
                                </div>
                            </section>

                            <section class="chart-panel" aria-labelledby="chart-title">
                                <div class="panel-header">
                                    <div><span class="panel-index">01</span><h2 id="chart-title">Price history</h2></div>
                                    <div class="range-picker" aria-label="Chart range">
                                        {RANGES.map(([value, label]) => (
                                            <button
                                                type="button"
                                                key={value}
                                                aria-pressed={range === value}
                                                onClick={() => setRange(value)}
                                            >
                                                {label}
                                            </button>
                                        ))}
                                    </div>
                                </div>
                                <PriceChart points={chart.points} range={range} currency={chart.currency} positive={positive} />
                                <div class="chart-caption">
                                    <span>{chart.points.length} observations</span>
                                    <span>{chart.interval} interval</span>
                                    <span>Provider may be delayed</span>
                                </div>
                            </section>

                            <section class="facts-section" aria-labelledby="facts-title">
                                <div class="panel-header"><div><span class="panel-index">02</span><h2 id="facts-title">Market facts</h2></div></div>
                                <dl class="facts-grid">
                                    <div><dt>Previous close</dt><dd>{formatPrice(chart.previousClose, chart.currency)}</dd></div>
                                    <div><dt>Day range</dt><dd>{chart.dayLow === null || chart.dayHigh === null ? '--' : `${chart.dayLow.toFixed(2)} - ${chart.dayHigh.toFixed(2)}`}</dd></div>
                                    <div><dt>52-week range</dt><dd>{chart.fiftyTwoWeekLow === null || chart.fiftyTwoWeekHigh === null ? '--' : `${chart.fiftyTwoWeekLow.toFixed(2)} - ${chart.fiftyTwoWeekHigh.toFixed(2)}`}</dd></div>
                                    <div><dt>Volume</dt><dd>{formatCompact(chart.volume)}</dd></div>
                                    <div><dt>Exchange</dt><dd>{chart.exchange}</dd></div>
                                    <div><dt>Instrument</dt><dd>{chart.instrumentType}</dd></div>
                                </dl>
                            </section>

                            <section class="discover-section" aria-labelledby="discover-title">
                                <div class="panel-header"><div><span class="panel-index">03</span><h2 id="discover-title">Global watchlist</h2></div><span class="section-note">Select an instrument to inspect</span></div>
                                <div class="market-grid">
                                    {marketTiles.map((item) => <MarketTile key={item.symbol} chart={item} onSelect={selectSymbol} />)}
                                </div>
                            </section>
                        </>
                    ) : (
                        <section class="loading-state" aria-live="polite">
                            <span class="loading-rule"></span>
                            <h1>Connecting to market data</h1>
                            <p>Loading {symbol} and its latest price history.</p>
                        </section>
                    )}
                </main>
            </div>
            <SyncMonitor />
        </div>
    );
}
