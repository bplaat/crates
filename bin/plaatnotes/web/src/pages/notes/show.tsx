/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Note } from '../../../src-gen/api.ts';
import { Link, route } from '../../router.tsx';
import { noteExtractTile } from '../../utils.ts';
import { $authToken } from '../../services/auth.service.ts';
import { NotesService } from '../../services/notes.service.ts';
import { useSignal } from '@preact/signals';
import { Navbar } from '../../components/navbar.tsx';

export function NotesShow({ note_id }: { note_id: string }) {
    const authToken = useSignal($authToken.value);
    const [note, setNote] = useState<Note | null>(null);
    const [error, setError] = useState<string>('');
    const [isSaving, setIsSaving] = useState<boolean>(false);

    useEffect(() => {
        const unsub = $authToken.subscribe((v) => (authToken.value = v));
        return () => unsub();
    }, []);

    // @ts-ignore
    useEffect(async () => {
        document.title = 'PlaatNotes - Note loading...';

        if (!authToken.value) {
            route('/auth/login');
            return;
        }

        const loadedNote = await NotesService.getInstance().getNote(note_id);
        if (loadedNote) {
            document.title = `PlaatNotes - ${noteExtractTile(loadedNote.body)}`;
            setNote(loadedNote);
        }
    }, [note_id, authToken.value]);

    async function updateNote(note: Note) {
        document.title = `PlaatNotes - ${noteExtractTile(note.body)}`;
        setIsSaving(true);
        const success = await NotesService.getInstance().updateNote(note.id, note.body);
        if (!success) {
            setError('Failed to save note');
        }
        setIsSaving(false);
    }

    async function deleteNote() {
        if (!confirm('Are you sure you want to delete this note?')) return;

        const success = await NotesService.getInstance().deleteNote(note_id);
        if (success) {
            route('/');
        } else {
            setError('Failed to delete note');
        }
    }

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container">
                    <h1 class="title">{note ? noteExtractTile(note.body) : 'Loading...'}</h1>
                    <div class="buttons">
                        <Link href="/" class="button">
                            <span class="icon">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
                                </svg>
                            </span>
                            <span>Back</span>
                        </Link>
                        <button class="button is-danger" onClick={deleteNote}>
                            <span class="icon">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M19,4H15.5L14.5,3H9.5L8.5,4H5V6H19M6,19A2,2 0 0,0 8,21H16A2,2 0 0,0 18,19V7H6V19Z" />
                                </svg>
                            </span>
                            <span>Delete</span>
                        </button>
                    </div>

                    {error && <div class="notification is-danger">{error}</div>}

                    {note ? (
                        <div class="field">
                            <textarea
                                class="textarea has-fixed-size"
                                value={note.body}
                                rows={20}
                                disabled={isSaving}
                                onInput={(e) => updateNote({ ...note, body: (e.target as HTMLTextAreaElement).value })}
                            />
                        </div>
                    ) : (
                        <p>Loading...</p>
                    )}
                </div>
            </section>
        </>
    );
}
