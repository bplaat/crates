/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { Chart } from '../api.ts';
import { formatPrice } from '../format.ts';

export function MarketTile({ chart, onSelect }: { chart: Chart; onSelect: (symbol: string) => void }) {
    const positive = chart.change >= 0;
    return (
        <button class="market-tile" type="button" onClick={() => onSelect(chart.symbol)}>
            <span class="market-tile-top"><strong>{chart.symbol}</strong><small>{chart.exchange}</small></span>
            <span class="market-tile-price">{formatPrice(chart.regularMarketPrice, chart.currency)}</span>
            <span class={positive ? 'change is-positive' : 'change is-negative'}>
                {positive ? '+' : ''}{chart.changePercent.toFixed(2)}%
            </span>
        </button>
    );
}

