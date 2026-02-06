/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link, route } from '../../router.tsx';
import { $authToken } from '../../services/auth.service.ts';
import { NotesService } from '../../services/notes.service.ts';
import { useSignal } from '@preact/signals';
import { Navbar } from '../../components/navbar.tsx';

export function NotesCreate() {
    const authToken = useSignal($authToken.value);
    const [body, setBody] = useState<string>('');
    const [error, setError] = useState<string>('');

    useEffect(() => {
        document.title = 'PlaatNotes - Create Note';
        const unsub = $authToken.subscribe((v) => (authToken.value = v));
        return () => unsub();
    }, []);

    async function saveNote(event: SubmitEvent) {
        event.preventDefault();
        if (!authToken.value) {
            route('/auth/login');
            return;
        }

        const noteId = await NotesService.getInstance().createNote(body);
        if (noteId) {
            route(`/notes/${noteId}`);
        } else {
            setError('Failed to create note');
        }
    }

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container">
                    <h1 class="title">Create Note</h1>
                    <div class="buttons">
                        <Link href="/" class="button">
                            <span class="icon">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
                                </svg>
                            </span>
                            <span>Back</span>
                        </Link>
                    </div>

                    {error && <div class="notification is-danger">{error}</div>}

                    <form onSubmit={saveNote}>
                        <div class="field">
                            <textarea
                                class="textarea has-fixed-size"
                                value={body}
                                rows={20}
                                onInput={(e) => setBody((e.target as HTMLTextAreaElement).value)}
                                required
                            />
                        </div>

                        <div class="field">
                            <button class="button is-link" type="submit">
                                <span class="icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                        <path d="M15,9H5V5H15M12,19A3,3 0 0,1 9,16A3,3 0 0,1 12,13A3,3 0 0,1 15,16A3,3 0 0,1 12,19M17,3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V7L17,3Z" />
                                    </svg>
                                </span>
                                <span>Save note</span>
                            </button>
                        </div>
                    </form>
                </div>
            </section>
        </>
    );
}
