/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ChartPoint } from '../api.ts';
import { formatAxisTime } from '../format.ts';

const WIDTH = 960;
const HEIGHT = 380;
const LEFT = 16;
const RIGHT = 74;
const TOP = 22;
const PRICE_BOTTOM = 284;
const VOLUME_TOP = 306;
const BOTTOM = 356;

interface PriceChartProps {
    points: ChartPoint[];
    range: string;
    currency: string;
    positive: boolean;
}

export function PriceChart({ points, range, currency, positive }: PriceChartProps) {
    if (points.length < 2) {
        return <div class="chart-empty">No chart data is available for this range.</div>;
    }

    const lows = points.map((point) => point.low);
    const highs = points.map((point) => point.high);
    const min = Math.min(...lows);
    const max = Math.max(...highs);
    const span = Math.max(max - min, Math.abs(max) * 0.005, 0.01);
    const paddedMin = min - span * 0.08;
    const paddedMax = max + span * 0.08;
    const plotWidth = WIDTH - LEFT - RIGHT;
    const priceHeight = PRICE_BOTTOM - TOP;
    const maxVolume = Math.max(...points.map((point) => point.volume), 1);
    const x = (index: number) => LEFT + (index / (points.length - 1)) * plotWidth;
    const y = (value: number) => TOP + ((paddedMax - value) / (paddedMax - paddedMin)) * priceHeight;
    const line = points.map((point, index) => `${x(index)},${y(point.close)}`).join(' ');
    const area = `${LEFT},${PRICE_BOTTOM} ${line} ${WIDTH - RIGHT},${PRICE_BOTTOM}`;
    const gridValues = Array.from({ length: 5 }, (_, index) => paddedMin + ((paddedMax - paddedMin) * index) / 4);
    const labelIndexes = Array.from(new Set([0, Math.floor((points.length - 1) / 3), Math.floor(((points.length - 1) * 2) / 3), points.length - 1]));
    const barWidth = Math.max(1, Math.min(6, (plotWidth / points.length) * 0.7));
    const trendClass = positive ? 'is-positive' : 'is-negative';

    return (
        <svg class={`price-chart ${trendClass}`} viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label={`${range} price and volume chart`}>
            <defs>
                <linearGradient id="price-area" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0" stop-color="currentColor" stop-opacity="0.2" />
                    <stop offset="1" stop-color="currentColor" stop-opacity="0" />
                </linearGradient>
            </defs>
            {gridValues.map((value) => (
                <g key={value}>
                    <line class="chart-grid" x1={LEFT} x2={WIDTH - RIGHT} y1={y(value)} y2={y(value)} />
                    <text class="chart-axis chart-axis-y" x={WIDTH - RIGHT + 10} y={y(value) + 4}>
                        {value.toLocaleString(undefined, { maximumFractionDigits: 2 })}
                    </text>
                </g>
            ))}
            <polygon class="chart-area" points={area} />
            <polyline class="chart-line" points={line} />
            {points.map((point, index) => {
                const barHeight = (point.volume / maxVolume) * (BOTTOM - VOLUME_TOP);
                return (
                    <rect
                        class="volume-bar"
                        key={point.time}
                        x={x(index) - barWidth / 2}
                        y={BOTTOM - barHeight}
                        width={barWidth}
                        height={barHeight}
                    />
                );
            })}
            {labelIndexes.map((index) => (
                <text
                    class="chart-axis"
                    key={points[index].time}
                    x={x(index)}
                    y={HEIGHT - 4}
                    text-anchor={index === 0 ? 'start' : index === points.length - 1 ? 'end' : 'middle'}
                >
                    {formatAxisTime(points[index].time, range)}
                </text>
            ))}
            <text class="chart-unit" x={WIDTH - RIGHT} y={14} text-anchor="end">
                {currency}
            </text>
        </svg>
    );
}

