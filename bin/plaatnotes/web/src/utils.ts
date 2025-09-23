/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export function noteExtractTile(body: string): string {
    const match = body.match(/^#\s*(.+)$/m);
    return match ? match[1].trim().replace(/\n.*/s, '') : 'Untitled';
}
