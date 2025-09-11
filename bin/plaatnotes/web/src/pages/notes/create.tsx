/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { Link, route } from '../../router.tsx';

export function NotesCreate() {
    const [body, setBody] = useState<string>('');

    async function saveNote() {
        const res = await fetch('/api/notes', {
            method: 'POST',
            body: new URLSearchParams({ body }),
        });
        if (res.status == 200) {
            const { id }: { id: string } = await res.json();
            route(`/notes/${id}`);
        }
    }

    return (
        <>
            <h1>PlaatNotes</h1>
            <p>
                <Link href="/">Back</Link>
            </p>
            <textarea value={body} onInput={(e) => setBody((e.target as HTMLTextAreaElement).value)} />
            <button onClick={() => saveNote()}>Save</button>
        </>
    );
}
