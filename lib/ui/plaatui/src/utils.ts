/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// Join truthy class names into a single className string. Typed loosely so it
// accepts plain strings, `cond && 'name'` short-circuits and preact's signal-ish
// `class` prop alike.
export function cx(...values: unknown[]): string {
    return values.filter(Boolean).join(' ');
}
