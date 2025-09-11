/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note, type NoteIndexResponse } from '../api.ts';
import { noteExtractTile } from '../utils.ts';
import { Link } from '../router.tsx';

export function Home() {
    const [notes, setNotes] = useState<Note[]>([]);

    useEffect(() => {
        fetch('/api/notes')
            .then((res) => res.json())
            .then(({ data }: NoteIndexResponse) => setNotes(data));
    }, []);

    async function deleteNote(id: string) {
        if (confirm('Are you sure you want to delete this note?')) {
            await fetch(`/api/notes/${id}`, { method: 'DELETE' });
            setNotes((notes) => notes.filter((note) => note.id !== id));
        }
    }

    return (
        <>
            <h1>PlaatNotes</h1>
            <p>
                <Link href="/notes/create">Create a new note</Link>
            </p>

            <ul>
                {notes.map((note) => (
                    <li key={note.id}>
                        <b>{noteExtractTile(note.body) || 'Untitled'}</b>

                        <Link href={`/notes/${note.id}`}>Edit</Link>
                        <a
                            href="#"
                            onClick={(e) => {
                                e.preventDefault();
                                deleteNote(note.id);
                            }}
                        >
                            Delete
                        </a>
                    </li>
                ))}
            </ul>
            {notes.length === 0 && (
                <p>
                    <i>No notes yet. Create one!</i>
                </p>
            )}
        </>
    );
}
