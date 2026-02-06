/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link } from '../../router.tsx';
import { $authToken } from '../../services/auth.service.ts';
import { NotesService } from '../../services/notes.service.ts';
import { useSignal } from '@preact/signals';
import type { Note, User } from '../../../src-gen/api.ts';
import { Navbar } from '../../components/navbar.tsx';

export function AdminNotes() {
    const authToken = useSignal($authToken.value);
    const [notes, setNotes] = useState<(Note & { user?: User })[]>([]);
    const [users, setUsers] = useState<User[]>([]);
    const [selectedUser, setSelectedUser] = useState<string>('');

    useEffect(() => {
        document.title = 'PlaatNotes - Admin Notes';
        const unsub = $authToken.subscribe((v) => (authToken.value = v));
        return () => unsub();
    }, []);

    // @ts-ignore
    useEffect(async () => {
        if (!authToken.value) return;

        const notesWithUsers = await NotesService.getInstance().getAllNotesWithUsers();
        setNotes(notesWithUsers);

        const userSet = new Set<string>();
        notesWithUsers.forEach((note) => {
            if (note.user?.id) {
                userSet.add(note.user.id);
            }
        });

        // Extract unique users from notes
        const uniqueUsers: User[] = [];
        notesWithUsers.forEach((note) => {
            if (note.user && !uniqueUsers.find((u) => u.id === note.user?.id)) {
                uniqueUsers.push(note.user);
            }
        });
        setUsers(uniqueUsers);
    }, [authToken.value]);

    async function deleteNote(id: string) {
        if (!confirm('Are you sure you want to delete this note?')) return;

        const success = await NotesService.getInstance().deleteNote(id);
        if (success) {
            setNotes(notes.filter((n) => n.id !== id));
        }
    }

    const filteredNotes = selectedUser ? notes.filter((n) => String(n.userId) === selectedUser) : notes;

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container">
                    <h1 class="title">All Notes</h1>
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

                    <div class="field">
                        <label class="label">Filter by User</label>
                        <div class="select">
                            <select
                                value={selectedUser}
                                onChange={(e) => setSelectedUser((e.target as HTMLSelectElement).value)}
                            >
                                <option value="">All Users</option>
                                {users.map((user) => (
                                    <option key={user.id} value={String(user.id)}>
                                        {user.firstName} {user.lastName} ({user.email})
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <table class="table is-fullwidth">
                        <thead>
                            <tr>
                                <th>Preview</th>
                                <th>Author</th>
                                <th>Created</th>
                                <th>Updated</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filteredNotes.map((note) => {
                                const preview = note.body.substring(0, 50).replace(/\n/g, ' ');
                                return (
                                    <tr key={note.id}>
                                        <td>{preview}...</td>
                                        <td>
                                            {note.user ? `${note.user.firstName} ${note.user.lastName}` : 'Unknown'}
                                        </td>
                                        <td>{new Date(note.createdAt).toLocaleDateString()}</td>
                                        <td>{new Date(note.updatedAt).toLocaleDateString()}</td>
                                        <td>
                                            <Link href={`/notes/${note.id}`} class="button is-small">
                                                View
                                            </Link>
                                            <button
                                                class="button is-small is-danger"
                                                onClick={() => deleteNote(note.id)}
                                            >
                                                Delete
                                            </button>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            </section>
        </>
    );
}
