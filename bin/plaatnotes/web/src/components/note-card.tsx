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
function NoteIconButton({ class: extraClass, ...props }: preact.JSX.IntrinsicElements['button']) {
    return <button type="button" {...props} class={extraClass ? `note-action ${extraClass}` : 'note-action'} />;
}

const NOTE_COLORS = ['is-default', 'is-yellow', 'is-green', 'is-blue', 'is-pink', 'is-purple', 'is-orange'];

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
        <Link href={`/notes/${note.id}`} class={`note ${noteColor(note.id)}${isDraggable ? ' is-draggable' : ''}`}>
            {title && <p class="note-title">{title}</p>}
            {snippetLines.length > 0 && <p class="note-snippet">{snippetLines.join('\n')}</p>}

            <div class="note-actions">
                {onPin && (
                    <NoteIconButton
                        class={note.isPinned ? 'is-pinned' : ''}
                        title={note.isPinned ? t('note.unpin') : t('note.pin')}
                        onClick={(e) => act(e, () => onPin(note))}
                    >
                        <PinIcon class="is-sm" />
                    </NoteIconButton>
                )}
                {onArchive && (
                    <NoteIconButton
                        class="is-hover-neutral"
                        title={t('note.archive')}
                        onClick={(e) => act(e, () => onArchive(note))}
                    >
                        <ArchiveArrowDownIcon class="is-sm" />
                    </NoteIconButton>
                )}
                {onUnarchive && (
                    <NoteIconButton
                        class="is-hover-neutral"
                        title={t('note.unarchive')}
                        onClick={(e) => act(e, () => onUnarchive(note))}
                    >
                        <ArchiveArrowUpIcon class="is-sm" />
                    </NoteIconButton>
                )}
                {onRestore && (
                    <NoteIconButton
                        class="is-hover-success"
                        title={t('note.restore')}
                        onClick={(e) => act(e, () => onRestore(note))}
                    >
                        <RestoreIcon class="is-sm" />
                    </NoteIconButton>
                )}
                {onTrash && (
                    <NoteIconButton
                        class="is-hover-danger"
                        title={t('note.trash')}
                        onClick={(e) => act(e, () => onTrash(note))}
                    >
                        <DeleteIcon class="is-sm" />
                    </NoteIconButton>
                )}
                {onDeleteForever && (
                    <NoteIconButton
                        class="is-hover-danger"
                        title={t('note.delete_forever')}
                        onClick={(e) => act(e, () => onDeleteForever(note))}
                    >
                        <DeleteOutlineIcon class="is-sm" />
                    </NoteIconButton>
                )}
            </div>
        </Link>
    );
}
