/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { $searchQuery } from '../services/notes.service.ts';

const DEBOUNCE_MS = 250;

// Returns the debounced search query for API fetching.
// - URL is updated immediately on every keystroke (replaceState)
// - The returned value only changes after DEBOUNCE_MS of no typing
// - On first mount the debounced value equals the current query (no delay on F5)
export function useSearchQuery(): string {
    const query = $searchQuery.value;
    const [debouncedQuery, setDebouncedQuery] = useState(query);

    // Sync signal → URL immediately
    useEffect(() => {
        const current = new URLSearchParams(window.location.search).get('q') ?? '';
        if (current === query) return;
        const params = new URLSearchParams();
        if (query) params.set('q', query);
        const search = params.toString();
        history.replaceState(null, '', window.location.pathname + (search ? `?${search}` : ''));
    }, [query]);

    // Debounce the value used for API fetching
    useEffect(() => {
        const timer = setTimeout(() => setDebouncedQuery(query), DEBOUNCE_MS);
        return () => clearTimeout(timer);
    }, [query]);

    return debouncedQuery;
}
