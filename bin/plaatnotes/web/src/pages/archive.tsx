/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { Layout } from '../components/layout.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { useSearchQuery } from '../hooks/use-search-query.ts';

export function ArchivePage() {
    const query = useSearchQuery();
    const { items, loading, hasMore, sentinelRef, setItems } = useInfiniteScroll(listArchivedNotes, query);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.archive')}`;
    }, []);

    async function handleUnarchive(note: Note) {
        await updateNote(note, { isArchived: false });
        setItems((ns) => ns.filter((n) => n.id !== note.id));
    }

    async function handleTrash(note: Note) {
        await updateNote(note, { isTrashed: true });
        setItems((ns) => ns.filter((n) => n.id !== note.id));
    }

    const notes = useMemo(
        () => items.slice().sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt)),
        [items],
    );

    return (
        <Layout showSearch>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                <h1 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-6">
                    {t('archive.heading')}
                </h1>

                {loading && items.length === 0 && <p class="text-center text-gray-400 mt-16">{t('archive.loading')}</p>}

                {!loading && notes.length === 0 && (
                    <EmptyState
                        icon={
                            <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                            </svg>
                        }
                        message={query ? t('archive.empty_search') : t('archive.empty')}
                    />
                )}

                {notes.length > 0 && (
                    <DraggableNoteGrid
                        notes={notes}
                        reorderEndpoint="/notes/archived/reorder"
                        onReorder={(reordered) => setItems(reordered)}
                        onUnarchive={handleUnarchive}
                        onTrash={handleTrash}
                    />
                )}

                {hasMore && <div ref={sentinelRef} class="h-1" />}
                {loading && items.length > 0 && <p class="text-center text-gray-400 py-4">{t('archive.loading')}</p>}
            </div>
        </Layout>
    );
}
