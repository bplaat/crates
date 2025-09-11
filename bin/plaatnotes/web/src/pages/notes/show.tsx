/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../api.ts';
import { Link } from '../../router.tsx';

export function NotesShow({ note_id }: { note_id: string }) {
    const [note, setNote] = useState<Note | null>(null);

    useEffect(() => {
        fetch(`/api/notes/${note_id}`)
            .then((res) => res.json())
            .then((note: Note) => setNote(note));
    }, [note_id]);

    async function updateNote(note: Note) {
        await fetch(`/api/notes/${note.id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: new URLSearchParams({ body: note.body }),
        });
    }

    return (
        <>
            <h1>PlaatNotes</h1>
            <p>
                <Link href="/">Back</Link>
            </p>

            {note ? (
                <textarea
                    value={note.body}
                    onInput={(e) => updateNote({ ...note, body: (e.target as HTMLTextAreaElement).value })}
                />
            ) : (
                <p>Loading...</p>
            )}
        </>
    );
}
