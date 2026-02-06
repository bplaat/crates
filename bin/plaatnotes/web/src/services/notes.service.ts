/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { API_URL } from '../consts.ts';
import type { Note, User } from '../../src-gen/api.ts';
import { $authToken } from './auth.service.ts';

export class NotesService {
    static instance?: NotesService;

    static getInstance(): NotesService {
        if (NotesService.instance === undefined) {
            NotesService.instance = new NotesService();
        }
        return NotesService.instance;
    }

    async getNotes(limit?: number, query?: string): Promise<Note[]> {
        try {
            const params = new URLSearchParams();
            if (limit) params.append('limit', String(limit));
            if (query) params.append('query', query);

            const url = params.toString() ? `${API_URL}/notes?${params.toString()}` : `${API_URL}/notes`;
            const res = await fetch(url, {
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });

            if (res.status !== 200) {
                return [];
            }
            const data = await res.json();
            return data.data || [];
        } catch {
            return [];
        }
    }

    async getAllNotes(): Promise<Note[]> {
        try {
            const res = await fetch(`${API_URL}/notes`, {
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });

            if (res.status !== 200) {
                return [];
            }
            const data = await res.json();
            return data.data || [];
        } catch {
            return [];
        }
    }

    async getNote(noteId: string): Promise<Note | null> {
        try {
            const res = await fetch(`${API_URL}/notes/${noteId}`, {
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });

            if (res.status !== 200) {
                return null;
            }
            return await res.json();
        } catch {
            return null;
        }
    }

    async createNote(body: string): Promise<string | null> {
        try {
            const res = await fetch(`${API_URL}/notes`, {
                method: 'POST',
                headers: { Authorization: `Bearer ${$authToken.value}` },
                body: new URLSearchParams({ body }),
            });

            if (res.status !== 200) {
                return null;
            }
            const { id }: { id: string } = await res.json();
            return id;
        } catch {
            return null;
        }
    }

    async updateNote(noteId: string, body: string): Promise<boolean> {
        try {
            const res = await fetch(`${API_URL}/notes/${noteId}`, {
                method: 'PUT',
                headers: { Authorization: `Bearer ${$authToken.value}` },
                body: new URLSearchParams({ body }),
            });

            return res.status === 200;
        } catch {
            return false;
        }
    }

    async deleteNote(noteId: string): Promise<boolean> {
        try {
            const res = await fetch(`${API_URL}/notes/${noteId}`, {
                method: 'DELETE',
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });

            return res.status === 200;
        } catch {
            return false;
        }
    }

    async getNoteWithUser(noteId: string): Promise<(Note & { user?: User }) | null> {
        try {
            const note = await this.getNote(noteId);
            if (!note) return null;

            try {
                const userRes = await fetch(`${API_URL}/users/${note.userId}`, {
                    headers: { Authorization: `Bearer ${$authToken.value}` },
                });
                if (userRes.status === 200) {
                    const user = await userRes.json();
                    return { ...note, user };
                }
            } catch {
                // Ignore user fetch errors
            }

            return note;
        } catch {
            return null;
        }
    }

    async getAllNotesWithUsers(): Promise<(Note & { user?: User })[]> {
        try {
            const notes = await this.getAllNotes();
            if (notes.length === 0) return [];

            const notesWithUsers = await Promise.all(
                notes.map(async (note) => {
                    try {
                        const userRes = await fetch(`${API_URL}/users/${note.userId}`, {
                            headers: { Authorization: `Bearer ${$authToken.value}` },
                        });
                        if (userRes.status === 200) {
                            const user = await userRes.json();
                            return { ...note, user };
                        }
                    } catch {
                        // Ignore user fetch errors
                    }
                    return note;
                }),
            );

            return notesWithUsers;
        } catch {
            return [];
        }
    }
}
