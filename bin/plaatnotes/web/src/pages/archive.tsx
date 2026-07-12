/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from 'plaatui';
import { AppLayout } from '../components/app-layout.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { Icon, LoadingText } from 'plaatui';
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
            <div class="page">
                <h1 class="section-label is-spaced">{t('archive.heading')}</h1>

                {loading && items.length === 0 && <LoadingText initial>{t('archive.loading')}</LoadingText>}

                {!loading && notes.length === 0 && (
                    <EmptyState
                        icon={<Icon type="package-down" class="is-huge" />}
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

                {hasMore && <div ref={sentinelRef} class="sentinel" />}
                {loading && items.length > 0 && <LoadingText>{t('archive.loading')}</LoadingText>}
            </div>
        </AppLayout>
    );
}
