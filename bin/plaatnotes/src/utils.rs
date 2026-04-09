/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) fn password_hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).expect("Can't get random bytes");
    pbkdf2::password_hash_customized(password, &salt, crate::consts::PBKDF2_ITERATIONS)
}

// Preprocess a user-supplied search string into a safe FTS5 query expression.
// FTS5 keyword operators (AND, OR, NOT) are preserved so callers can use them
// intentionally. Structural characters that cause FTS5 parse errors (unmatched
// quotes and parentheses) are stripped from individual tokens.
pub(crate) fn preprocess_fts_query(q: &str) -> String {
    const FTS5_KEYWORDS: &[&str] = &["AND", "OR", "NOT"];

    let tokens: Vec<_> = q.split_whitespace().collect();
    let mut parts: Vec<String> = Vec::with_capacity(tokens.len());

    for token in &tokens {
        if FTS5_KEYWORDS.contains(token) {
            // Keep recognized operators as-is
            parts.push(token.to_string());
        } else {
            // Strip characters that cause FTS5 parse errors: unmatched quotes and parens
            let clean: String = token
                .chars()
                .filter(|c| !matches!(c, '"' | '(' | ')'))
                .collect();
            if !clean.is_empty() {
                // Preserve explicit trailing * (user-supplied prefix wildcard),
                // otherwise add one for prefix matching
                if clean.ends_with('*') {
                    parts.push(clean);
                } else {
                    parts.push(format!("{clean}*"));
                }
            }
        }
    }

    // Remove dangling operators at the start or end which cause FTS5 syntax errors
    while parts
        .first()
        .is_some_and(|p| FTS5_KEYWORDS.contains(&p.as_str()))
    {
        parts.remove(0);
    }
    while parts
        .last()
        .is_some_and(|p| FTS5_KEYWORDS.contains(&p.as_str()))
    {
        parts.pop();
    }

    parts.join(" ")
}
