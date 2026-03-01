/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { Layout } from '../components/layout.tsx';
import { NoteCard } from '../components/note-card.tsx';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';

export function ArchivePage() {
    const [notes, setNotes] = useState<Note[]>([]);
    const [loading, setLoading] = useState(true);

    // @ts-ignore
    useEffect(async () => {
        document.title = `PlaatNotes - ${t('page.archive')}`;
        const data = await listArchivedNotes();
        setNotes(data);
        setLoading(false);
    }, []);

    async function handleUnarchive(id: string) {
        await updateNote(id, { isArchived: false });
        setNotes((ns) => ns.filter((n) => n.id !== id));
    }

    async function handleTrash(id: string) {
        await updateNote(id, { isTrashed: true });
        setNotes((ns) => ns.filter((n) => n.id !== id));
    }

    return (
        <Layout>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                <h1 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-6">
                    {t('archive.heading')}
                </h1>

                {loading && <p class="text-center text-gray-400 mt-16">{t('archive.loading')}</p>}

                {!loading && notes.length === 0 && (
                    <div class="flex flex-col items-center justify-center mt-24 gap-3 text-gray-400">
                        <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                        </svg>
                        <p class="text-lg">{t('archive.empty')}</p>
                    </div>
                )}

                {notes.length > 0 && (
                    <div class="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4">
                        {notes.map((note) => (
                            <NoteCard key={note.id} note={note} onUnarchive={handleUnarchive} onTrash={handleTrash} />
                        ))}
                    </div>
                )}
            </div>
        </Layout>
    );
}
