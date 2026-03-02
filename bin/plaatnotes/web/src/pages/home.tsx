/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { Layout } from '../components/layout.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { listNotes, listPinnedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';

export function Home() {
    const {
        items: pinnedItems,
        loading: pinnedLoading,
        hasMore: pinnedHasMore,
        sentinelRef: pinnedSentinelRef,
        setItems: setPinnedItems,
    } = useInfiniteScroll(listPinnedNotes);
    const {
        items: otherItems,
        loading: otherLoading,
        hasMore: otherHasMore,
        sentinelRef: otherSentinelRef,
        setItems: setOtherItems,
    } = useInfiniteScroll(listNotes);

    useEffect(() => {
        document.title = 'PlaatNotes';
    }, []);

    async function handlePin(note: Note) {
        const updated = await updateNote(note, { isPinned: !note.isPinned });
        if (updated.isPinned) {
            setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
            setPinnedItems((ns) => [...ns, updated]);
        } else {
            setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
            setOtherItems((ns) => [...ns, updated]);
        }
    }

    async function handleArchive(note: Note) {
        await updateNote(note, { isArchived: true });
        setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
        setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
    }

    async function handleTrash(note: Note) {
        await updateNote(note, { isTrashed: true });
        setPinnedItems((ns) => ns.filter((n) => n.id !== note.id));
        setOtherItems((ns) => ns.filter((n) => n.id !== note.id));
    }

    const initialLoading = (pinnedLoading && pinnedItems.length === 0) || (otherLoading && otherItems.length === 0);
    const isEmpty = !pinnedLoading && !otherLoading && pinnedItems.length === 0 && otherItems.length === 0;

    const pinned = pinnedItems
        .slice()
        .sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt));
    const others = otherItems.slice().sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt));

    return (
        <Layout>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                {initialLoading && <p class="text-center text-gray-400 mt-16">{t('home.loading')}</p>}

                {isEmpty && (
                    <EmptyState
                        icon={
                            <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
                            </svg>
                        }
                        message={t('home.empty')}
                    />
                )}

                {pinned.length > 0 && (
                    <section class="mb-6">
                        <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">
                            {t('home.pinned')}
                        </h2>
                        <DraggableNoteGrid
                            notes={pinned}
                            reorderEndpoint="/notes/pinned/reorder"
                            onReorder={setPinnedItems}
                            onPin={handlePin}
                            onArchive={handleArchive}
                            onTrash={handleTrash}
                        />
                        {pinnedHasMore && <div ref={pinnedSentinelRef} class="h-1" />}
                        {pinnedLoading && pinnedItems.length > 0 && (
                            <p class="text-center text-gray-400 py-4">{t('home.loading')}</p>
                        )}
                    </section>
                )}

                {others.length > 0 && (
                    <section>
                        {pinned.length > 0 && (
                            <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">
                                {t('home.other')}
                            </h2>
                        )}
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

                {otherHasMore && <div ref={otherSentinelRef} class="h-1" />}
                {otherLoading && otherItems.length > 0 && (
                    <p class="text-center text-gray-400 py-4">{t('home.loading')}</p>
                )}
            </div>

            <button
                onClick={() => route('/notes/create')}
                class="fixed bottom-6 right-6 w-14 h-14 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 text-white rounded-full shadow-lg flex items-center justify-center transition-colors cursor-pointer"
                title={t('home.create')}
            >
                <svg class="w-7 h-7" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
                </svg>
            </button>
        </Layout>
    );
}
