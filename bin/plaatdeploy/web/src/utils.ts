/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export function capitalizeLabel(value: string) {
    return value ? `${value[0].toUpperCase()}${value.slice(1)}` : value;
}
