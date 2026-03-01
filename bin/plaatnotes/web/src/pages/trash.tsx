/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { Layout } from '../components/layout.tsx';
import { NoteCard } from '../components/note-card.tsx';
import { deleteNote, listNotes, updateNote } from '../services/notes.service.ts';

export function TrashPage() {
    const [notes, setNotes] = useState<Note[]>([]);
    const [loading, setLoading] = useState(true);

    // @ts-ignore
    useEffect(async () => {
        document.title = 'PlaatNotes - Trash';
        const data = await listNotes();
        setNotes(data.filter((n) => n.isTrashed));
        setLoading(false);
    }, []);

    async function handleRestore(id: string) {
        await updateNote(id, { isTrashed: false });
        setNotes((ns) => ns.filter((n) => n.id !== id));
    }

    async function handleDeleteForever(id: string) {
        if (confirm('Delete this note forever? This cannot be undone.')) {
            await deleteNote(id);
            setNotes((ns) => ns.filter((n) => n.id !== id));
        }
    }

    async function handleEmptyTrash() {
        if (confirm('Empty trash? All trashed notes will be deleted forever.')) {
            await Promise.all(notes.map((n) => deleteNote(n.id)));
            setNotes([]);
        }
    }

    return (
        <Layout>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-xs font-semibold uppercase tracking-wider text-gray-400">Trash</h1>
                    {notes.length > 0 && (
                        <button
                            onClick={handleEmptyTrash}
                            class="text-sm text-red-400 hover:text-red-600 transition-colors cursor-pointer"
                        >
                            Empty trash
                        </button>
                    )}
                </div>

                {!loading && notes.length > 0 && (
                    <p class="text-xs text-gray-400 mb-4">
                        Notes in trash will be permanently deleted after some time.
                    </p>
                )}

                {loading && <p class="text-center text-gray-400 mt-16">Loading…</p>}

                {!loading && notes.length === 0 && (
                    <div class="flex flex-col items-center justify-center mt-24 gap-3 text-gray-400">
                        <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                        </svg>
                        <p class="text-lg">Trash is empty.</p>
                    </div>
                )}

                {notes.length > 0 && (
                    <div class="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4">
                        {notes.map((note) => (
                            <NoteCard
                                key={note.id}
                                note={note}
                                onRestore={handleRestore}
                                onDeleteForever={handleDeleteForever}
                            />
                        ))}
                    </div>
                )}
            </div>
        </Layout>
    );
}
