/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useRef } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { reorderNotes } from '../services/notes.service.ts';
import { NoteCard } from './note-card.tsx';

interface DraggableNoteGridProps {
    notes: Note[];
    onReorder: (notes: Note[]) => void;
    onPin?: (note: Note) => void;
    onArchive?: (note: Note) => void;
    onUnarchive?: (note: Note) => void;
    onTrash?: (note: Note) => void;
    onRestore?: (note: Note) => void;
    onDeleteForever?: (note: Note) => void;
}

export function DraggableNoteGrid({
    notes,
    onReorder,
    onPin,
    onArchive,
    onUnarchive,
    onTrash,
    onRestore,
    onDeleteForever,
}: DraggableNoteGridProps) {
    const dragId = useRef<string | null>(null);
    const dragOverId = useRef<string | null>(null);

    function handleDragStart(e: DragEvent, note: Note) {
        dragId.current = note.id;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', note.id);
        }
    }

    function handleDragOver(e: DragEvent, note: Note) {
        e.preventDefault();
        if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
        dragOverId.current = note.id;
    }

    function handleDrop(e: DragEvent) {
        e.preventDefault();
        const fromId = dragId.current;
        const toId = dragOverId.current;
        dragId.current = null;
        dragOverId.current = null;
        if (!fromId || !toId || fromId === toId) return;

        const fromIdx = notes.findIndex((n) => n.id === fromId);
        const toIdx = notes.findIndex((n) => n.id === toId);
        if (fromIdx === -1 || toIdx === -1) return;

        const reordered = [...notes];
        const [moved] = reordered.splice(fromIdx, 1);
        reordered.splice(toIdx, 0, moved);

        // Assign new positions
        const updated = reordered.map((n, i) => ({ ...n, position: i }));
        onReorder(updated);
        void reorderNotes(updated.map((n) => n.id));
    }

    return (
        <div class="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4">
            {notes.map((note) => (
                <div
                    key={note.id}
                    draggable
                    onDragStart={(e) => handleDragStart(e, note)}
                    onDragOver={(e) => handleDragOver(e, note)}
                    onDrop={handleDrop}
                >
                    <NoteCard
                        note={note}
                        draggable
                        onPin={onPin}
                        onArchive={onArchive}
                        onUnarchive={onUnarchive}
                        onTrash={onTrash}
                        onRestore={onRestore}
                        onDeleteForever={onDeleteForever}
                    />
                </div>
            ))}
        </div>
    );
}
