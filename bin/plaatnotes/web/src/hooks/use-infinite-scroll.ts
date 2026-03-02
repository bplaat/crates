/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useRef, useState } from 'preact/hooks';
import { type Pagination } from '../../src-gen/api.ts';

export function useInfiniteScroll<T>(
    fetchFn: (page: number, query?: string) => Promise<{ data: T[]; pagination: Pagination }>,
    query = '',
) {
    const [items, setItems] = useState<T[]>([]);
    const [loading, setLoading] = useState(true);
    const [hasMore, setHasMore] = useState(false);
    const pageRef = useRef(1);
    const sentinelRef = useRef<HTMLDivElement>(null);
    const fetchingRef = useRef(false);

    async function fetchPage(page: number) {
        if (fetchingRef.current) return;
        fetchingRef.current = true;
        setLoading(true);
        const { data, pagination } = await fetchFn(page, query);
        setItems((prev) => (page === 1 ? data : [...prev, ...data]));
        setHasMore(page * pagination.limit < pagination.total);
        pageRef.current = page;
        setLoading(false);
        fetchingRef.current = false;
    }

    useEffect(() => {
        setHasMore(false);
        void fetchPage(1);
    }, [query]);

    useEffect(() => {
        const el = sentinelRef.current;
        if (!el || !hasMore) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && !fetchingRef.current) {
                    void fetchPage(pageRef.current + 1);
                }
            },
            { rootMargin: '0px 0px 20% 0px' },
        );
        observer.observe(el);
        return () => observer.disconnect();
    }, [hasMore]);

    return { items, loading, hasMore, sentinelRef, setItems };
}
