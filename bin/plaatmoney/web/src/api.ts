/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export interface ChartPoint {
    time: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
}

export interface Chart {
    symbol: string;
    name: string;
    exchange: string;
    instrumentType: string;
    currency: string;
    timezone: string;
    regularMarketPrice: number;
    previousClose: number;
    change: number;
    changePercent: number;
    marketTime: number;
    dayHigh: number | null;
    dayLow: number | null;
    volume: number | null;
    fiftyTwoWeekHigh: number | null;
    fiftyTwoWeekLow: number | null;
    range: string;
    interval: string;
    provider: string;
    points: ChartPoint[];
}

export interface SearchResult {
    symbol: string;
    name: string;
    exchange: string;
    instrumentType: string;
    sector: string | null;
    industry: string | null;
}

export interface SyncStatus {
    phase: string;
    status: string;
    current: number;
    total: number;
    cursor: string | null;
    message: string;
    errorCount: number;
    updatedAt: number;
    instrumentCount: number;
    historySyncedCount: number;
    storedBarCount: number;
}

export async function getChart(symbol: string, range: string, signal?: AbortSignal): Promise<Chart> {
    const response = await fetch(`/api/chart/${encodeURIComponent(symbol)}?range=${encodeURIComponent(range)}`, {
        signal,
    });
    if (!response.ok) throw new Error('Quote data is unavailable');
    return response.json();
}

export async function searchTickers(query: string, signal?: AbortSignal): Promise<SearchResult[]> {
    const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`, { signal });
    if (!response.ok) throw new Error('Search is unavailable');
    return response.json();
}
