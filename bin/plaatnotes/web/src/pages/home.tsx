/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useLocation } from 'wouter-preact';
import { useEffect, useMemo } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { AppLayout } from '../components/app-layout.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { listNotes, listPinnedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { NoteTextIcon, PlusIcon } from '../components/icons.tsx';
import { useSearchQuery } from '../hooks/use-search-query.ts';

export function Home() {
    const query = useSearchQuery();
    const [, navigate] = useLocation();
    const {
        items: pinnedItems,
        loading: pinnedLoading,
        hasMore: pinnedHasMore,
        sentinelRef: pinnedSentinelRef,
        setItems: setPinnedItems,
    } = useInfiniteScroll(listPinnedNotes, query);
    const {
        items: otherItems,
        loading: otherLoading,
        hasMore: otherHasMore,
        sentinelRef: otherSentinelRef,
        setItems: setOtherItems,
    } = useInfiniteScroll(listNotes, query);

    useEffect(() => {
        document.title = 'PlaatNotes';
    }, []);

    async function handlePin(note: Note) {
        // Optimistic update
        if (note.isPinned) {
            setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
            setOtherItems((ns) => [{ ...note, isPinned: false }, ...ns]);
        } else {
            setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
            setPinnedItems((ns) => [...ns, { ...note, isPinned: true }]);
        }
        const updated = await updateNote(note, { isPinned: !note.isPinned });
        // Rollback if the server rejected the change (service returns original note on error)
        if (updated.isPinned === note.isPinned) {
            if (note.isPinned) {
                setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
                setPinnedItems((ns) => [...ns, note]);
            } else {
                setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
                setOtherItems((ns) => [...ns, note]);
            }
        }
    }

    async function handleArchive(note: Note) {
        // Optimistic update
        setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
        setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
        const updated = await updateNote(note, { isArchived: true });
        // Rollback on failure
        if (!updated.isArchived) {
            if (note.isPinned) setPinnedItems((ns) => [...ns, note]);
            else setOtherItems((ns) => [...ns, note]);
        }
    }

    async function handleTrash(note: Note) {
        // Optimistic update
        setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
        setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
        const updated = await updateNote(note, { isTrashed: true });
        // Rollback on failure
        if (!updated.isTrashed) {
            if (note.isPinned) setPinnedItems((ns) => [...ns, note]);
            else setOtherItems((ns) => [...ns, note]);
        }
    }

    const initialLoading = pinnedLoading && pinnedItems.length === 0 && otherLoading && otherItems.length === 0;
    const isEmpty = !pinnedLoading && !otherLoading && pinnedItems.length === 0 && otherItems.length === 0;

    const pinned = useMemo(
        () => pinnedItems.slice().sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt)),
        [pinnedItems],
    );
    const others = useMemo(
        () => otherItems.slice().sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt)),
        [otherItems],
    );

    return (
        <AppLayout showSearch>
            <div class="page">
                {initialLoading && <p class="loading-text is-initial">{t('home.loading')}</p>}

                {isEmpty && (
                    <EmptyState
                        icon={<NoteTextIcon class="is-huge" />}
                        message={query ? t('home.empty_search') : t('home.empty')}
                    />
                )}

                {pinned.length > 0 && (
                    <section class="note-section">
                        <h2 class="section-label">{t('home.pinned')}</h2>
                        <DraggableNoteGrid
                            notes={pinned}
                            reorderEndpoint="/notes/pinned/reorder"
                            onReorder={setPinnedItems}
                            onPin={handlePin}
                            onArchive={handleArchive}
                            onTrash={handleTrash}
                        />
                        {pinnedHasMore && <div ref={pinnedSentinelRef} class="sentinel" />}
                        {pinnedLoading && pinnedItems.length > 0 && <p class="loading-text">{t('home.loading')}</p>}
                    </section>
                )}

                {others.length > 0 && (
                    <section>
                        {pinned.length > 0 && <h2 class="section-label">{t('home.other')}</h2>}
                        <DraggableNoteGrid
                            notes={others}
                            reorderEndpoint="/notes/reorder"
                            onReorder={setOtherItems}
                            onPin={handlePin}
                            onArchive={handleArchive}
                            onTrash={handleTrash}
                        />
                    </section>
                )}

                {otherHasMore && <div ref={otherSentinelRef} class="sentinel" />}
                {otherLoading && otherItems.length > 0 && <p class="loading-text">{t('home.loading')}</p>}
            </div>

            <button onClick={() => navigate('/notes/create')} class="fab" title={t('home.create')}>
                <PlusIcon class="is-xl" />
            </button>
        </AppLayout>
    );
}
