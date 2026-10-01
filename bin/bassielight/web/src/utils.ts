/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export function capitalize(string: string): string {
    return string.charAt(0).toUpperCase() + string.slice(1);
}

export function colorToHex(color: number): string {
    return `#${color.toString(16).padStart(6, '0')}`;
}

export function clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max);
}
