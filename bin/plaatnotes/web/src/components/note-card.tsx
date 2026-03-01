/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link } from 'preact-router';
import { type Note } from '../../src-gen/api.ts';
import { t } from '../services/i18n.service.ts';

const NOTE_COLORS = [
    'bg-white dark:bg-zinc-800',
    'bg-yellow-100 dark:bg-yellow-900/40',
    'bg-green-100 dark:bg-green-900/40',
    'bg-blue-100 dark:bg-blue-900/40',
    'bg-pink-100 dark:bg-pink-900/40',
    'bg-purple-100 dark:bg-purple-900/40',
    'bg-orange-100 dark:bg-orange-900/40',
];

function noteColor(id: string): string {
    let hash = 0;
    for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) | 0;
    return NOTE_COLORS[Math.abs(hash) % NOTE_COLORS.length];
}

interface NoteCardProps {
    note: Note;
    onPin?: (note: Note) => void;
    onArchive?: (id: string) => void;
    onUnarchive?: (id: string) => void;
    onTrash?: (id: string) => void;
    onRestore?: (id: string) => void;
    onDeleteForever?: (id: string) => void;
}

export function NoteCard({ note, onPin, onArchive, onUnarchive, onTrash, onRestore, onDeleteForever }: NoteCardProps) {
    const lines = note.body.split('\n').filter(Boolean);
    const title = note.title || (lines[0]?.startsWith('#') ? lines[0].replace(/^#+\s*/, '') : null);
    const bodyLines = lines.filter((l, i) => !(i === 0 && l.startsWith('#'))).slice(0, 8);

    function act(e: MouseEvent, cb?: () => void) {
        e.preventDefault();
        e.stopPropagation();
        cb?.();
    }

    return (
        <Link
            href={`/notes/${note.id}`}
            class={`block rounded-xl border border-gray-200 dark:border-zinc-600 hover:border-gray-400 dark:hover:border-gray-400 hover:shadow-md transition-all cursor-pointer p-4 mb-4 break-inside-avoid group no-underline ${noteColor(note.id)}`}
        >
            {title && <p class="font-medium text-gray-800 dark:text-gray-100 mb-1 truncate">{title}</p>}
            {bodyLines.length > 0 && (
                <p class="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-wrap line-clamp-6">
                    {bodyLines.join('\n')}
                </p>
            )}

            <div class="flex items-center justify-end gap-0.5 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
                {onPin && (
                    <button
                        class={`p-1.5 rounded-full hover:bg-black/10 transition-colors cursor-pointer ${note.isPinned ? 'text-yellow-500' : 'text-gray-400'}`}
                        title={note.isPinned ? t('note.unpin') : t('note.pin')}
                        onClick={(e) => act(e, () => onPin(note))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
                        </svg>
                    </button>
                )}
                {onArchive && (
                    <button
                        class="p-1.5 rounded-full hover:bg-black/10 text-gray-400 hover:text-gray-700 transition-colors cursor-pointer"
                        title={t('note.archive')}
                        onClick={(e) => act(e, () => onArchive(note.id))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                        </svg>
                    </button>
                )}
                {onUnarchive && (
                    <button
                        class="p-1.5 rounded-full hover:bg-black/10 text-gray-400 hover:text-gray-700 transition-colors cursor-pointer"
                        title={t('note.unarchive')}
                        onClick={(e) => act(e, () => onUnarchive(note.id))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20.55 5.22l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.15.55L3.46 5.22C3.17 5.57 3 6.01 3 6.5V19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.49-.17-.93-.45-1.28zM12 9.5l5.5 5.5H14v2h-4v-2H6.5L12 9.5zM5.12 5l.82-1h12l.93 1H5.12z" />
                        </svg>
                    </button>
                )}
                {onRestore && (
                    <button
                        class="p-1.5 rounded-full hover:bg-black/10 text-gray-400 hover:text-green-600 transition-colors cursor-pointer"
                        title={t('note.restore')}
                        onClick={(e) => act(e, () => onRestore(note.id))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M13 3a9 9 0 0 0-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0 0 13 21a9 9 0 0 0 0-18z" />
                        </svg>
                    </button>
                )}
                {onTrash && (
                    <button
                        class="p-1.5 rounded-full hover:bg-black/10 text-gray-400 hover:text-red-500 transition-colors cursor-pointer"
                        title={t('note.trash')}
                        onClick={(e) => act(e, () => onTrash(note.id))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 4h-3.5l-1-1h-5l-1 1H5v2h14M6 19a2 2 0 002 2h8a2 2 0 002-2V7H6v12z" />
                        </svg>
                    </button>
                )}
                {onDeleteForever && (
                    <button
                        class="p-1.5 rounded-full hover:bg-black/10 text-gray-400 hover:text-red-700 transition-colors cursor-pointer"
                        title={t('note.delete_forever')}
                        onClick={(e) => act(e, () => onDeleteForever(note.id))}
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                        </svg>
                    </button>
                )}
            </div>
        </Link>
    );
}
