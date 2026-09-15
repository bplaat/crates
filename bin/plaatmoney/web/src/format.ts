/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export function formatPrice(value: number, currency: string): string {
    try {
        return new Intl.NumberFormat(undefined, {
            style: 'currency',
            currency,
            minimumFractionDigits: value >= 100 ? 2 : 3,
            maximumFractionDigits: value >= 100 ? 2 : 3,
        }).format(value);
    } catch {
        return `${currency} ${value.toFixed(2)}`;
    }
}

export function formatCompact(value: number | null): string {
    if (value === null) return '--';
    return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 2 }).format(value);
}

export function formatDateTime(timestamp: number): string {
    if (!timestamp) return 'Unknown';
    return new Intl.DateTimeFormat(undefined, {
        day: 'numeric',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit',
    }).format(new Date(timestamp * 1000));
}

export function formatAxisTime(timestamp: number, range: string): string {
    const date = new Date(timestamp * 1000);
    if (range === '1d' || range === '5d') {
        return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(date);
    }
    return new Intl.DateTimeFormat(undefined, { month: 'short', year: range === '5y' ? '2-digit' : undefined }).format(
        date,
    );
}

