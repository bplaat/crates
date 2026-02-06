/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../src-gen/api.ts';
import { noteExtractTile } from '../utils.ts';
import { Link } from '../router.tsx';
import { $authToken } from '../services/auth.service.ts';
import { NotesService } from '../services/notes.service.ts';
import { useSignal } from '@preact/signals';
import { Navbar } from '../components/navbar.tsx';

export function Home() {
    const authToken = useSignal($authToken.value);
    const [notes, setNotes] = useState<Note[]>([]);

    useEffect(() => {
        const unsub = $authToken.subscribe((v) => (authToken.value = v));
        return () => unsub();
    }, []);

    // @ts-ignore
    useEffect(async () => {
        document.title = 'PlaatNotes';

        if (!authToken.value) return;

        const loadedNotes = await NotesService.getInstance().getNotes();
        setNotes(loadedNotes);
    }, [authToken.value]);

    async function deleteNote(id: string) {
        if (confirm('Are you sure you want to delete this note?')) {
            const success = await NotesService.getInstance().deleteNote(id);
            if (success) {
                setNotes((notes) => notes.filter((note) => note.id !== id));
            }
        }
    }

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container">
                    <h1 class="title">Your Notes</h1>

                    <div class="buttons">
                        <Link href="/notes/create" class="button is-link">
                            <span class="icon">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M19,13H13V19H11V13H5V11H11V5H13V11H19V13Z" />
                                </svg>
                            </span>
                            <span>Create a new note</span>
                        </Link>
                    </div>

                    {notes.length > 0 ? (
                        <div class="fixed-grid has-3-cols">
                            <div class="grid">
                                {notes.map((note) => (
                                    <Link href={`/notes/${note.id}`} class="box" key={note.id}>
                                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
                                            <b>{noteExtractTile(note.body)}</b>
                                            <button
                                                class="delete"
                                                onClick={(e) => {
                                                    e.preventDefault();
                                                    e.stopPropagation();
                                                    deleteNote(note.id);
                                                }}
                                            />
                                        </div>
                                    </Link>
                                ))}
                            </div>
                        </div>
                    ) : (
                        <div class="box">
                            <p>
                                <i>No notes yet. Create one!</i>
                            </p>
                        </div>
                    )}
                </div>
            </section>
        </>
    );
}
