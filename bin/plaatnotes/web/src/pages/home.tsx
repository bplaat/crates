/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note, type NoteIndexResponse } from '../api.ts';
import { noteExtractTile } from '../utils.ts';
import { Link } from '../router.tsx';
import { API_URL } from '../consts.ts';

export function Home() {
    const [notes, setNotes] = useState<Note[]>([]);

    useEffect(() => {
        fetch(`${API_URL}/notes`)
            .then((res) => res.json())
            .then(({ data }: NoteIndexResponse) => setNotes(data));
    }, []);

    async function deleteNote(id: string) {
        if (confirm('Are you sure you want to delete this note?')) {
            await fetch(`${API_URL}/notes/${id}`, { method: 'DELETE' });
            setNotes((notes) => notes.filter((note) => note.id !== id));
        }
    }

    return (
        <div class="container">
            <h1 class="title">PlaatNotes</h1>
            <div class="buttons">
                <Link href="/notes/create" class="button is-link">
                    Create a new note
                </Link>
            </div>

            <div class="fixed-grid has-3-cols">
                <div class="grid">
                    {notes.map((note) => (
                        <Link href={`/notes/${note.id}`} class="box" key={note.id}>
                            <b>{noteExtractTile(note.body) || 'Untitled'}</b>

                            <button
                                class="delete is-pulled-right"
                                onClick={(e) => {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    deleteNote(note.id);
                                }}
                            />
                        </Link>
                    ))}
                </div>
            </div>
            {notes.length === 0 && (
                <p>
                    <i>No notes yet. Create one!</i>
                </p>
            )}

            <p>
                Made by <a href="https://bplaat.nl">Bastiaan van der Plaat</a>
            </p>
        </div>
    );
}
