/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link } from 'wouter-preact';
import { type Note } from '../../src-gen/api.ts';
import {
    ArchiveArrowDownIcon,
    ArchiveArrowUpIcon,
    DeleteIcon,
    DeleteOutlineIcon,
    PinIcon,
    RestoreIcon,
} from './icons.tsx';
import { t } from '../services/i18n.service.ts';

// Icon button for use on colored card backgrounds - uses semi-transparent hover overlay
const CARD_BTN_BASE = 'p-2 rounded-full hover:bg-black/10 dark:hover:bg-white/15 transition-colors cursor-pointer';

function NoteIconButton({ class: extraClass, ...props }: preact.JSX.IntrinsicElements['button']) {
    return <button type="button" {...props} class={extraClass ? `${CARD_BTN_BASE} ${extraClass}` : CARD_BTN_BASE} />;
}

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
    draggable?: boolean;
    onPin?: (note: Note) => void;
    onArchive?: (note: Note) => void;
    onUnarchive?: (note: Note) => void;
    onTrash?: (note: Note) => void;
    onRestore?: (note: Note) => void;
    onDeleteForever?: (note: Note) => void;
}

function stripMarkdown(text: string): string {
    return text
        .replace(/<[^>]*>/g, '') // strip HTML tags (e.g. <br>, <u>)
        .replace(/^#{1,6}\s+/gm, '') // headings
        .replace(/\*\*(.+?)\*\*/g, '$1') // bold
        .replace(/\*(.+?)\*/g, '$1') // italic
        .replace(/__(.+?)__/g, '$1') // bold alt
        .replace(/_(.+?)_/g, '$1') // italic alt
        .replace(/~~(.+?)~~/g, '$1') // strikethrough
        .replace(/`{3}[\s\S]*?`{3}/g, '') // fenced code blocks
        .replace(/`(.+?)`/g, '$1') // inline code
        .replace(/^\s*[-*+]\s+/gm, '') // unordered list bullets
        .replace(/^\s*\d+\.\s+/gm, '') // ordered list numbers
        .replace(/^\s*>\s*/gm, '') // blockquotes
        .replace(/^---+$/gm, '') // horizontal rules
        .replace(/\[(.+?)\]\(.+?\)/g, '$1'); // links
}

export function NoteCard({
    note,
    draggable: isDraggable,
    onPin,
    onArchive,
    onUnarchive,
    onTrash,
    onRestore,
    onDeleteForever,
}: NoteCardProps) {
    const lines = note.body.split('\n').filter(Boolean);
    const title = note.title || (lines[0]?.startsWith('#') ? lines[0].replace(/^#+\s*/, '') : null);
    const snippetLines = lines.map(stripMarkdown).filter(Boolean).slice(0, 8);

    function act(e: MouseEvent, cb?: () => void) {
        e.preventDefault();
        e.stopPropagation();
        cb?.();
    }

    return (
        <Link
            href={`/notes/${note.id}`}
            class={`block rounded-xl border border-gray-200 dark:border-zinc-600 hover:border-gray-400 dark:hover:border-gray-400 hover:shadow-md transition-all p-4 mb-4 break-inside-avoid group no-underline ${noteColor(note.id)}${isDraggable ? ' cursor-grab active:cursor-grabbing' : ' cursor-pointer'}`}
        >
            {title && <p class="font-medium text-gray-800 dark:text-gray-100 mb-1 truncate">{title}</p>}
            {snippetLines.length > 0 && (
                <p class="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-wrap line-clamp-6">
                    {snippetLines.join('\n')}
                </p>
            )}

            <div class="flex items-center justify-end gap-0.5 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
                {onPin && (
                    <NoteIconButton
                        class={
                            note.isPinned ? 'text-yellow-500 dark:text-yellow-400' : 'text-gray-400 dark:text-gray-500'
                        }
                        title={note.isPinned ? t('note.unpin') : t('note.pin')}
                        onClick={(e) => act(e, () => onPin(note))}
                    >
                        <PinIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
                {onArchive && (
                    <NoteIconButton
                        class="text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-200"
                        title={t('note.archive')}
                        onClick={(e) => act(e, () => onArchive(note))}
                    >
                        <ArchiveArrowDownIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
                {onUnarchive && (
                    <NoteIconButton
                        class="text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-200"
                        title={t('note.unarchive')}
                        onClick={(e) => act(e, () => onUnarchive(note))}
                    >
                        <ArchiveArrowUpIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
                {onRestore && (
                    <NoteIconButton
                        class="text-gray-400 dark:text-gray-500 hover:text-green-600 dark:hover:text-green-400"
                        title={t('note.restore')}
                        onClick={(e) => act(e, () => onRestore(note))}
                    >
                        <RestoreIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
                {onTrash && (
                    <NoteIconButton
                        class="text-gray-400 dark:text-gray-500 hover:text-red-500 dark:hover:text-red-400"
                        title={t('note.trash')}
                        onClick={(e) => act(e, () => onTrash(note))}
                    >
                        <DeleteIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
                {onDeleteForever && (
                    <NoteIconButton
                        class="text-gray-400 dark:text-gray-500 hover:text-red-700 dark:hover:text-red-500"
                        title={t('note.delete_forever')}
                        onClick={(e) => act(e, () => onDeleteForever(note))}
                    >
                        <DeleteOutlineIcon class="w-4 h-4" />
                    </NoteIconButton>
                )}
            </div>
        </Link>
    );
}
