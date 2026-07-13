/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { EmptyState, LoadingText, PackageDownIcon, Page, SectionLabel } from 'plaatui';
import { useEffect, useMemo } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { AppLayout } from '../components/app-layout.tsx';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { useSearchQuery } from '../hooks/use-search-query.ts';
import { t } from '../services/i18n.service.ts';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';

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
            <Page>
                <SectionLabel as="h1" spaced>
                    {t('archive.heading')}
                </SectionLabel>

                {loading && items.length === 0 && <LoadingText initial>{t('archive.loading')}</LoadingText>}

                {!loading && notes.length === 0 && (
                    <EmptyState
                        icon={<PackageDownIcon class="is-huge" />}
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
            </Page>
        </AppLayout>
    );
}
