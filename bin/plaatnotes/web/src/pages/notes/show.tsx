/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useRef, useState } from 'preact/hooks';
import { type Note } from '../../../src-gen/api.ts';
import { Navbar } from '../../components/navbar.tsx';
import { getNote, updateNote } from '../../services/notes.service.ts';
import { formatDate, t } from '../../services/i18n.service.ts';

export function NotesShow({ note_id }: { note_id?: string }) {
    const [note, setNote] = useState<Note | null>(null);
    const saveTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

    // @ts-ignore
    useEffect(async () => {
        document.title = `PlaatNotes - ${t('page.note_loading')}`;
        const loaded = await getNote(note_id!);
        document.title = `PlaatNotes - ${loaded.title || loaded.body.slice(0, 40)}`;
        setNote(loaded);
    }, [note_id]);

    function scheduleSave(updated: Note) {
        setNote(updated);
        if (saveTimeout.current) clearTimeout(saveTimeout.current);
        saveTimeout.current = setTimeout(async () => {
            const saved = await updateNote(updated.id, { body: updated.body, title: updated.title });
            setNote((current) => (current ? { ...current, updatedAt: saved.updatedAt } : current));
        }, 600);
    }

    async function handleArchive() {
        if (!note) return;
        await updateNote(note.id, { isArchived: !note.isArchived });
        route(note.isArchived ? '/' : '/archive');
    }

    async function handleTrash() {
        if (!note) return;
        await updateNote(note.id, { isTrashed: !note.isTrashed });
        route(note.isTrashed ? '/' : '/trash');
    }

    async function handlePin() {
        if (!note) return;
        const updated = await updateNote(note.id, { isPinned: !note.isPinned });
        setNote(updated);
    }

    if (!note) {
        return (
            <div class="min-h-screen bg-gray-50 dark:bg-zinc-900">
                <Navbar />
                <p class="text-center text-gray-400 dark:text-gray-500 mt-24">{t('app.loading')}</p>
            </div>
        );
    }

    return (
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900">
            <Navbar />
            <main class="max-w-2xl mx-auto px-4 py-8">
                <div class="flex items-center gap-3 mb-6">
                    <button
                        onClick={() => route('/')}
                        class="p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 text-gray-500 dark:text-gray-400 transition-colors cursor-pointer"
                        title={t('notes_show.back')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
                        </svg>
                    </button>
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">{t('notes_show.heading')}</h1>
                    <div class="flex-1" />
                    <button
                        onClick={handlePin}
                        class={`p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors cursor-pointer ${note.isPinned ? 'text-yellow-500' : 'text-gray-400'}`}
                        title={note.isPinned ? t('note.unpin') : t('note.pin')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
                        </svg>
                    </button>
                    <button
                        onClick={handleArchive}
                        class={`p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors cursor-pointer ${note.isArchived ? 'text-yellow-600' : 'text-gray-400'}`}
                        title={note.isArchived ? t('note.unarchive') : t('note.archive')}
                    >
                        {note.isArchived ? (
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.55 5.22l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.15.55L3.46 5.22C3.17 5.57 3 6.01 3 6.5V19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.49-.17-.93-.45-1.28zM12 9.5l5.5 5.5H14v2h-4v-2H6.5L12 9.5zM5.12 5l.82-1h12l.93 1H5.12z" />
                            </svg>
                        ) : (
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                            </svg>
                        )}
                    </button>
                    <button
                        onClick={handleTrash}
                        class={`p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors cursor-pointer ${note.isTrashed ? 'text-red-500' : 'text-gray-400'}`}
                        title={note.isTrashed ? t('note.restore') : t('note.trash')}
                    >
                        {note.isTrashed ? (
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M13 3a9 9 0 0 0-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0 0 13 21a9 9 0 0 0 0-18z" />
                            </svg>
                        ) : (
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                            </svg>
                        )}
                    </button>
                </div>

                <div class="bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm overflow-hidden">
                    <div class="p-5 flex flex-col gap-4">
                        <input
                            class="text-xl font-medium text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-600 bg-transparent outline-none w-full"
                            type="text"
                            placeholder={t('notes_show.title_placeholder')}
                            value={note.title || ''}
                            onInput={(e) => scheduleSave({ ...note, title: (e.target as HTMLInputElement).value })}
                        />
                        <textarea
                            class="text-gray-700 dark:text-gray-300 bg-transparent outline-none w-full resize-none min-h-96 font-mono text-sm"
                            placeholder={t('notes_show.body_placeholder')}
                            value={note.body}
                            rows={20}
                            onInput={(e) => scheduleSave({ ...note, body: (e.target as HTMLTextAreaElement).value })}
                        />
                    </div>
                    <div class="border-t border-gray-100 dark:border-zinc-700 px-5 py-2 bg-gray-50 dark:bg-zinc-700/50">
                        <p class="text-xs text-gray-400 dark:text-gray-500">
                            {t('notes_show.last_updated', formatDate(note.updatedAt))}
                        </p>
                    </div>
                </div>
            </main>
        </div>
    );
}
