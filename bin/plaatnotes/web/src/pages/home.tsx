/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { Layout } from '../components/layout.tsx';
import { NoteCard } from '../components/note-card.tsx';
import { listNotes, updateNote } from '../services/notes.service.ts';

export function Home() {
    const [notes, setNotes] = useState<Note[]>([]);
    const [loading, setLoading] = useState(true);

    // @ts-ignore
    useEffect(async () => {
        document.title = 'PlaatNotes';
        const data = await listNotes();
        setNotes(data);
        setLoading(false);
    }, []);

    async function handlePin(note: Note) {
        const updated = await updateNote(note.id, { isPinned: !note.isPinned });
        setNotes((ns) => ns.map((n) => (n.id === note.id ? updated : n)));
    }

    async function handleArchive(id: string) {
        await updateNote(id, { isArchived: true });
        setNotes((ns) => ns.filter((n) => n.id !== id));
    }

    async function handleTrash(id: string) {
        await updateNote(id, { isTrashed: true });
        setNotes((ns) => ns.filter((n) => n.id !== id));
    }

    const active = notes.filter((n) => !n.isArchived && !n.isTrashed);
    const pinned = active.filter((n) => n.isPinned);
    const others = active.filter((n) => !n.isPinned);

    return (
        <Layout>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                {loading && <p class="text-center text-gray-400 mt-16">Loading notes…</p>}

                {!loading && active.length === 0 && (
                    <div class="flex flex-col items-center justify-center mt-24 gap-3 text-gray-400">
                        <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
                        </svg>
                        <p class="text-lg">No notes yet. Create one!</p>
                    </div>
                )}

                {pinned.length > 0 && (
                    <section class="mb-6">
                        <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">Pinned</h2>
                        <div class="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4">
                            {pinned.map((note) => (
                                <NoteCard
                                    key={note.id}
                                    note={note}
                                    onPin={handlePin}
                                    onArchive={handleArchive}
                                    onTrash={handleTrash}
                                />
                            ))}
                        </div>
                    </section>
                )}

                {others.length > 0 && (
                    <section>
                        {pinned.length > 0 && (
                            <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">Other</h2>
                        )}
                        <div class="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4">
                            {others.map((note) => (
                                <NoteCard
                                    key={note.id}
                                    note={note}
                                    onPin={handlePin}
                                    onArchive={handleArchive}
                                    onTrash={handleTrash}
                                />
                            ))}
                        </div>
                    </section>
                )}
            </div>

            <button
                onClick={() => route('/notes/create')}
                class="fixed bottom-6 right-6 w-14 h-14 bg-yellow-400 hover:bg-yellow-500 text-white rounded-full shadow-lg flex items-center justify-center transition-colors cursor-pointer"
                title="Create note"
            >
                <svg class="w-7 h-7" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
                </svg>
            </button>
        </Layout>
    );
}
