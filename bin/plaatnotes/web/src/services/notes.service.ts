/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Note, type NoteIndexResponse } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';
import { authFetch } from './auth.service.ts';

export async function listNotes(): Promise<Note[]> {
    const res = await authFetch(`${API_URL}/notes`);
    const { data }: NoteIndexResponse = await res.json();
    return data;
}

export async function createNote(params: { body: string; title?: string; isPinned?: boolean }): Promise<Note> {
    const form = new URLSearchParams({ body: params.body });
    if (params.title) form.set('title', params.title);
    if (params.isPinned !== undefined) form.set('isPinned', String(params.isPinned));
    const res = await authFetch(`${API_URL}/notes`, { method: 'POST', body: form });
    return res.json();
}

export async function getNote(id: string): Promise<Note> {
    const res = await authFetch(`${API_URL}/notes/${id}`);
    return res.json();
}

export async function updateNote(
    id: string,
    params: { body?: string; title?: string; isPinned?: boolean; isArchived?: boolean; isTrashed?: boolean },
): Promise<Note> {
    const note = await getNote(id);
    const form = new URLSearchParams({
        body: params.body ?? note.body,
        isPinned: String(params.isPinned ?? note.isPinned),
        isArchived: String(params.isArchived ?? note.isArchived),
        isTrashed: String(params.isTrashed ?? note.isTrashed),
    });
    if (params.title !== undefined) form.set('title', params.title);
    else if (note.title) form.set('title', note.title);
    const res = await authFetch(`${API_URL}/notes/${id}`, { method: 'PUT', body: form });
    return res.json();
}

export async function deleteNote(id: string): Promise<void> {
    await authFetch(`${API_URL}/notes/${id}`, { method: 'DELETE' });
}
