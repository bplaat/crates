/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// Preprocess a user-supplied search string into an FTS5 query expression.
pub(crate) fn preprocess_fts_query(q: &str) -> String {
    // If the query already uses FTS5 syntax, pass it through unchanged
    if q.contains(" AND ")
        || q.contains(" OR ")
        || q.contains(" NOT ")
        || q.contains('"')
        || q.contains('(')
        || q.contains(')')
        || q.contains('*')
        || q.contains('-')
    {
        return q.to_string();
    }

    // Otherwise wrap each whitespace-separated token with a trailing * for prefix matching
    q.split_whitespace()
        .map(|token| format!("{token}*"))
        .collect::<Vec<_>>()
        .join(" OR ")
}
