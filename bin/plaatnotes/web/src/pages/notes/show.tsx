/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../api.ts';
import { Link } from '../../router.tsx';
import { API_URL } from '../../consts.ts';

export function NotesShow({ note_id }: { note_id: string }) {
    const [note, setNote] = useState<Note | null>(null);

    useEffect(() => {
        fetch(`${API_URL}/notes/${note_id}`)
            .then((res) => res.json())
            .then((note: Note) => setNote(note));
    }, [note_id]);

    async function updateNote(note: Note) {
        await fetch(`${API_URL}/notes/${note.id}`, {
            method: 'PUT',
            body: new URLSearchParams({ body: note.body }),
        });
    }

    return (
        <div class="container">
            <h1 class="title">PlaatNotes</h1>
            <div class="buttons">
                <Link href="/" class="button">
                    <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
                    </svg>
                    Back
                </Link>
            </div>

            {note ? (
                <div class="field">
                    <textarea
                        class="textarea has-fixed-size"
                        value={note.body}
                        rows={20}
                        onInput={(e) => updateNote({ ...note, body: (e.target as HTMLTextAreaElement).value })}
                    />
                </div>
            ) : (
                <p>Loading...</p>
            )}
        </div>
    );
}
