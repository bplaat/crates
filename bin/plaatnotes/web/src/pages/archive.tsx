/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { AppLayout } from '../components/app-layout.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { ArchiveArrowDownIcon } from '../components/icons.tsx';
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
        <AppLayout showSearch>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                <h1 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-6">
                    {t('archive.heading')}
                </h1>

                {loading && items.length === 0 && <p class="text-center text-gray-400 mt-16">{t('archive.loading')}</p>}

                {!loading && notes.length === 0 && (
                    <EmptyState
                        icon={<ArchiveArrowDownIcon class="w-16 h-16" />}
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
        </AppLayout>
    );
}
