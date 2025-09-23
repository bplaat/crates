/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link, route } from '../../router.tsx';
import { API_URL } from '../../consts.ts';

export function NotesCreate() {
    const [body, setBody] = useState<string>('');

    useEffect(() => {
        document.title = 'PlaatNotes - Create Note';
    }, []);

    async function saveNote(event: SubmitEvent) {
        event.preventDefault();
        const res = await fetch(`${API_URL}/notes`, {
            method: 'POST',
            body: new URLSearchParams({ body }),
        });
        if (res.status == 200) {
            const { id }: { id: string } = await res.json();
            route(`/notes/${id}`);
        }
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

            <form class="form" onSubmit={saveNote}>
                <div class="field">
                    <textarea
                        class="textarea has-fixed-size"
                        value={body}
                        rows={20}
                        onInput={(e) => setBody((e.target as HTMLTextAreaElement).value)}
                    />
                </div>

                <div class="field">
                    <button class="button is-link" type="submit">
                        <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                            <path d="M15,9H5V5H15M12,19A3,3 0 0,1 9,16A3,3 0 0,1 12,13A3,3 0 0,1 15,16A3,3 0 0,1 12,19M17,3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V7L17,3Z" />
                        </svg>
                        Save note
                    </button>
                </div>
            </form>
        </div>
    );
}
