/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useState } from 'preact/hooks';
import { Navbar } from '../../components/navbar.tsx';
import { createNote } from '../../services/notes.service.ts';

export function NotesCreate() {
    const [title, setTitle] = useState('');
    const [body, setBody] = useState('');
    const [isPinned, setIsPinned] = useState(false);

    useEffect(() => {
        document.title = 'PlaatNotes - Create Note';
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
        <div class="min-h-screen bg-gray-50">
            <Navbar />
            <main class="max-w-2xl mx-auto px-4 py-8">
                <div class="flex items-center gap-3 mb-6">
                    <button
                        onClick={() => route('/')}
                        class="p-2 rounded-full hover:bg-gray-200 text-gray-500 transition-colors cursor-pointer"
                        title="Back"
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
                        </svg>
                    </button>
                    <h1 class="text-xl font-medium text-gray-700">New note</h1>
                </div>

                <form onSubmit={saveNote} class="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
                    <div class="p-5 flex flex-col gap-4">
                        <input
                            class="text-xl font-medium text-gray-800 placeholder-gray-400 outline-none w-full"
                            type="text"
                            placeholder="Title"
                            value={title}
                            onInput={(e) => setTitle((e.target as HTMLInputElement).value)}
                        />
                        <textarea
                            class="text-gray-700 placeholder-gray-400 outline-none w-full resize-none min-h-48 font-mono text-sm"
                            placeholder="Take a note…"
                            required
                            value={body}
                            rows={12}
                            onInput={(e) => setBody((e.target as HTMLTextAreaElement).value)}
                        />
                    </div>

                    <div class="border-t border-gray-100 px-5 py-3 flex items-center justify-between bg-gray-50">
                        <button
                            type="button"
                            onClick={() => setIsPinned(!isPinned)}
                            class={`p-2 rounded-full hover:bg-gray-200 transition-colors cursor-pointer ${isPinned ? 'text-yellow-500' : 'text-gray-400'}`}
                            title={isPinned ? 'Unpin' : 'Pin'}
                        >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
                            </svg>
                        </button>
                        <div class="flex gap-2">
                            <button
                                type="button"
                                onClick={() => route('/')}
                                class="px-4 py-1.5 rounded-lg text-sm text-gray-600 hover:bg-gray-200 transition-colors cursor-pointer"
                            >
                                Cancel
                            </button>
                            <button
                                type="submit"
                                class="px-4 py-1.5 rounded-lg text-sm bg-yellow-400 hover:bg-yellow-500 text-white font-medium transition-colors cursor-pointer"
                            >
                                Save
                            </button>
                        </div>
                    </div>
                </form>
            </main>
        </div>
    );
}
