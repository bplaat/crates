/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useRef, useState } from 'preact/hooks';
import type { SearchResult } from '../api.ts';
import { searchTickers } from '../api.ts';

export function SearchBox({ onSelect }: { onSelect: (symbol: string) => void }) {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult[]>([]);
    const [open, setOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const container = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (query.trim().length < 1) {
            setResults([]);
            setOpen(false);
            return;
        }
        const controller = new AbortController();
        const timeout = window.setTimeout(() => {
            setLoading(true);
            searchTickers(query.trim(), controller.signal)
                .then((nextResults) => {
                    setResults(nextResults);
                    setOpen(true);
                })
                .catch((error: unknown) => {
                    if (error instanceof DOMException && error.name === 'AbortError') return;
                    setResults([]);
                    setOpen(true);
                })
                .finally(() => setLoading(false));
        }, 250);
        return () => {
            window.clearTimeout(timeout);
            controller.abort();
        };
    }, [query]);

    useEffect(() => {
        function close(event: PointerEvent) {
            if (!container.current?.contains(event.target as Node)) setOpen(false);
        }
        document.addEventListener('pointerdown', close);
        return () => document.removeEventListener('pointerdown', close);
    }, []);

    function choose(symbol: string) {
        setQuery('');
        setOpen(false);
        onSelect(symbol);
    }

    function submit(event: SubmitEvent) {
        event.preventDefault();
        if (results[0]) choose(results[0].symbol);
        else if (query.trim()) choose(query.trim().toUpperCase());
    }

    return (
        <div class="search-shell" ref={container}>
            <form class="search-form" role="search" onSubmit={submit}>
                <span class="search-glyph" aria-hidden="true">/</span>
                <label class="sr-only" for="ticker-search">Search stocks, ETFs and indices</label>
                <input
                    id="ticker-search"
                    value={query}
                    onInput={(event) => setQuery(event.currentTarget.value)}
                    onFocus={() => results.length > 0 && setOpen(true)}
                    placeholder="Search ticker, company or index"
                    autocomplete="off"
                />
                <span class="search-state">{loading ? 'Searching' : 'Enter'}</span>
            </form>
            {open && (
                <div class="search-results">
                    {results.length === 0 ? (
                        <div class="search-empty">No matching instruments found.</div>
                    ) : (
                        results.map((result) => (
                            <button type="button" key={`${result.symbol}-${result.exchange}`} onClick={() => choose(result.symbol)}>
                                <span class="result-symbol">{result.symbol}</span>
                                <span class="result-name">{result.name}</span>
                                <span class="result-exchange">{result.exchange}</span>
                            </button>
                        ))
                    )}
                </div>
            )}
        </div>
    );
}

