/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useState } from 'preact/hooks';
import { Navbar } from '../../components/navbar.tsx';
import { createNote } from '../../services/notes.service.ts';
import { t } from '../../services/i18n.service.ts';

export function NotesCreate() {
    const [title, setTitle] = useState('');
    const [body, setBody] = useState('');
    const [isPinned, setIsPinned] = useState(false);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.create')}`;
    }, []);

    async function saveNote(event: SubmitEvent) {
        event.preventDefault();
        if (!body.trim()) return;
        const note = await createNote({ body, title: title || undefined, isPinned });
        if (note?.id) {
            route(`/notes/${note.id}`);
        }
    }

    return (
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900">
            <Navbar />
            <main class="max-w-2xl mx-auto px-4 py-8">
                <div class="flex items-center gap-3 mb-6">
                    <button
                        onClick={() => route('/')}
                        class="p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 text-gray-500 dark:text-gray-400 transition-colors cursor-pointer"
                        title={t('notes_create.back')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
                        </svg>
                    </button>
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">{t('notes_create.heading')}</h1>
                </div>

                <form
                    onSubmit={saveNote}
                    class="bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm overflow-hidden"
                >
                    <div class="p-5 flex flex-col gap-4">
                        <input
                            class="text-xl font-medium text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-600 bg-transparent outline-none w-full"
                            type="text"
                            placeholder={t('notes_create.title_placeholder')}
                            value={title}
                            onInput={(e) => setTitle((e.target as HTMLInputElement).value)}
                        />
                        <textarea
                            class="text-gray-700 dark:text-gray-300 placeholder-gray-400 dark:placeholder-gray-600 bg-transparent outline-none w-full resize-none min-h-48 font-mono text-sm"
                            placeholder={t('notes_create.body_placeholder')}
                            required
                            value={body}
                            rows={12}
                            onInput={(e) => setBody((e.target as HTMLTextAreaElement).value)}
                        />
                    </div>

                    <div class="border-t border-gray-100 dark:border-zinc-700 px-5 py-3 flex items-center justify-between bg-gray-50 dark:bg-zinc-700/50">
                        <button
                            type="button"
                            onClick={() => setIsPinned(!isPinned)}
                            class={`p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-600 transition-colors cursor-pointer ${isPinned ? 'text-yellow-500' : 'text-gray-400'}`}
                            title={isPinned ? t('note.unpin') : t('note.pin')}
                        >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
                            </svg>
                        </button>
                        <div class="flex gap-2">
                            <button
                                type="button"
                                onClick={() => route('/')}
                                class="px-4 py-1.5 rounded-lg text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-zinc-600 transition-colors cursor-pointer"
                            >
                                {t('notes_create.cancel')}
                            </button>
                            <button
                                type="submit"
                                class="px-4 py-1.5 rounded-lg text-sm bg-yellow-400 hover:bg-yellow-500 text-white font-medium transition-colors cursor-pointer"
                            >
                                {t('notes_create.save')}
                            </button>
                        </div>
                    </div>
                </form>
            </main>
        </div>
    );
}
