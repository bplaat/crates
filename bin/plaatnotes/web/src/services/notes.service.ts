/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { type Note, type NoteCreateBody, type NoteIndexResponse, type NoteUpdateBody } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';
import { authFetch } from './auth.service.ts';

export const $notesCache = signal<Map<string, Note>>(new Map());

function cacheNotes(notes: Note[]) {
    $notesCache.value = new Map([...$notesCache.value, ...notes.map((n) => [n.id, n] as [string, Note])]);
}

export async function listNotes(): Promise<Note[]> {
    const res = await authFetch(`${API_URL}/notes`);
    const { data }: NoteIndexResponse = await res.json();
    cacheNotes(data);
    return data;
}

export async function listArchivedNotes(): Promise<Note[]> {
    const res = await authFetch(`${API_URL}/notes/archived`);
    const { data }: NoteIndexResponse = await res.json();
    cacheNotes(data);
    return data;
}

export async function listTrashedNotes(): Promise<Note[]> {
    const res = await authFetch(`${API_URL}/notes/trashed`);
    const { data }: NoteIndexResponse = await res.json();
    cacheNotes(data);
    return data;
}

export async function createNote(params: NoteCreateBody): Promise<Note> {
    const form = new URLSearchParams({ body: params.body });
    if (params.title) form.set('title', params.title);
    if (params.isPinned !== undefined) form.set('isPinned', String(params.isPinned));
    const res = await authFetch(`${API_URL}/notes`, { method: 'POST', body: form });
    const note: Note = await res.json();
    cacheNotes([note]);
    return note;
}

export async function getNote(id: string): Promise<Note> {
    if ($notesCache.value.has(id)) return $notesCache.value.get(id)!;
    const res = await authFetch(`${API_URL}/notes/${id}`);
    const note: Note = await res.json();
    cacheNotes([note]);
    return note;
}

export async function updateNote(note: Note, params: Partial<NoteUpdateBody>): Promise<Note> {
    const form = new URLSearchParams({
        body: params.body ?? note.body,
        isPinned: String(params.isPinned ?? note.isPinned),
        isArchived: String(params.isArchived ?? note.isArchived),
        isTrashed: String(params.isTrashed ?? note.isTrashed),
    });
    const title = params.title !== undefined ? params.title : note.title;
    if (title) form.set('title', title);
    const res = await authFetch(`${API_URL}/notes/${note.id}`, { method: 'PUT', body: form });
    const saved: Note = await res.json();
    cacheNotes([saved]);
    return saved;
}

export async function deleteNote(id: string): Promise<void> {
    await authFetch(`${API_URL}/notes/${id}`, { method: 'DELETE' });
    const next = new Map($notesCache.value);
    next.delete(id);
    $notesCache.value = next;
}
