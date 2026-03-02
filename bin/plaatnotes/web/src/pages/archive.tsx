/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { DraggableNoteGrid } from '../components/draggable-note-grid.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { Layout } from '../components/layout.tsx';
import { listArchivedNotes, updateNote } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';

export function ArchivePage() {
    const [notes, setNotes] = useState<Note[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        void (async () => {
            document.title = `PlaatNotes - ${t('page.archive')}`;
            const data = await listArchivedNotes();
            setNotes(data.slice().sort((a, b) => a.position - b.position || a.updatedAt.localeCompare(b.updatedAt)));
            setLoading(false);
        })();
    }, []);

    async function handleUnarchive(note: Note) {
        await updateNote(note, { isArchived: false });
        setNotes((ns) => ns.filter((n) => n.id !== note.id));
    }

    async function handleTrash(note: Note) {
        await updateNote(note, { isTrashed: true });
        setNotes((ns) => ns.filter((n) => n.id !== note.id));
    }

    return (
        <Layout>
            <div class="max-w-screen-xl mx-auto px-4 py-6">
                <h1 class="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-6">
                    {t('archive.heading')}
                </h1>

                {loading && <p class="text-center text-gray-400 mt-16">{t('archive.loading')}</p>}

                {!loading && notes.length === 0 && (
                    <EmptyState
                        icon={
                            <svg class="w-16 h-16" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                            </svg>
                        }
                        message={t('archive.empty')}
                    />
                )}

                {notes.length > 0 && (
                    <DraggableNoteGrid
                        notes={notes}
                        onReorder={setNotes}
                        onUnarchive={handleUnarchive}
                        onTrash={handleTrash}
                    />
                )}
            </div>
        </Layout>
    );
}
