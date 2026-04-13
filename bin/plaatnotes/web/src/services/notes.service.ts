/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
export const $searchQuery = signal('');

import {
    type Note,
    type NoteCreateBody,
    type NoteIndexResponse,
    type NoteUpdateBody,
    type Pagination,
} from '../../src-gen/api.ts';

import { authFetch } from './auth.service.ts';

const NOTE_CACHE_MAX_SIZE = 256;
export const $notesCache = signal<Map<string, Note>>(new Map());

function cacheNotes(notes: Note[]) {
    const merged = new Map([...$notesCache.value, ...notes.map((n) => [n.id, n] as [string, Note])]);
    // Evict oldest entries when cache exceeds max size
    if (merged.size > NOTE_CACHE_MAX_SIZE) {
        const excess = merged.size - NOTE_CACHE_MAX_SIZE;
        const keys = [...merged.keys()];
        for (let i = 0; i < excess; i++) merged.delete(keys[i]);
    }
    $notesCache.value = merged;
}

const emptyPage = (page: number): { data: Note[]; pagination: Pagination } => ({
    data: [],
    pagination: { page, limit: 20, total: 0 },
});

function buildUrl(base: string, page: number, query?: string): string {
    const q = query ? `&q=${encodeURIComponent(query)}` : '';
    return `${base}?page=${page}${q}`;
}

export async function listNotes(
    page = 1,
    query?: string,
    signal?: AbortSignal,
): Promise<{ data: Note[]; pagination: Pagination }> {
    const res = await authFetch(buildUrl(`/api/notes`, page, query), { signal });
    if (!res.ok) return emptyPage(page);
    const result: NoteIndexResponse = await res.json();
    cacheNotes(result.data);
    return result;
}

export async function listPinnedNotes(
    page = 1,
    query?: string,
    signal?: AbortSignal,
): Promise<{ data: Note[]; pagination: Pagination }> {
    const res = await authFetch(buildUrl(`/api/notes/pinned`, page, query), { signal });
    if (!res.ok) return emptyPage(page);
    const result: NoteIndexResponse = await res.json();
    cacheNotes(result.data);
    return result;
}

export async function listArchivedNotes(
    page = 1,
    query?: string,
    signal?: AbortSignal,
): Promise<{ data: Note[]; pagination: Pagination }> {
    const res = await authFetch(buildUrl(`/api/notes/archived`, page, query), { signal });
    if (!res.ok) return emptyPage(page);
    const result: NoteIndexResponse = await res.json();
    cacheNotes(result.data);
    return result;
}

export async function listTrashedNotes(
    page = 1,
    query?: string,
    signal?: AbortSignal,
): Promise<{ data: Note[]; pagination: Pagination }> {
    const res = await authFetch(buildUrl(`/api/notes/trashed`, page, query), { signal });
    if (!res.ok) return emptyPage(page);
    const result: NoteIndexResponse = await res.json();
    cacheNotes(result.data);
    return result;
}

export async function createNote(params: NoteCreateBody): Promise<Note | null> {
    const form = new URLSearchParams({ body: params.body });
    if (params.title) form.set('title', params.title);
    if (params.isPinned !== undefined) form.set('isPinned', String(params.isPinned));
    const res = await authFetch(`/api/notes`, { method: 'POST', body: form });
    if (!res.ok) return null;
    const note: Note = await res.json();
    cacheNotes([note]);
    return note;
}

export async function getNote(id: string): Promise<Note | null> {
    if ($notesCache.value.has(id)) return $notesCache.value.get(id)!;
    const res = await authFetch(`/api/notes/${id}`);
    if (!res.ok) return null;
    const note: Note = await res.json();
    cacheNotes([note]);
    return note;
}

export async function fetchNote(id: string): Promise<Note | null> {
    const res = await authFetch(`/api/notes/${id}`);
    if (!res.ok) return null;
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
    const res = await authFetch(`/api/notes/${note.id}`, { method: 'PUT', body: form });
    if (!res.ok) return note;
    const saved: Note = await res.json();
    cacheNotes([saved]);
    return saved;
}

export async function deleteNote(id: string): Promise<void> {
    await authFetch(`/api/notes/${id}`, { method: 'DELETE' });
    const next = new Map($notesCache.value);
    next.delete(id);
    $notesCache.value = next;
}

export async function clearTrashedNotes(): Promise<void> {
    await authFetch(`/api/notes/trashed/clear`, { method: 'DELETE' });
    $notesCache.value = new Map([...$notesCache.value].filter(([, note]) => !note.isTrashed));
}

export async function reorderNotes(ids: string[], endpoint: string): Promise<void> {
    const res = await authFetch(`/api${endpoint}`, {
        method: 'PUT',
        body: new URLSearchParams({ ids: ids.join(',') }),
    });
    if (!res.ok) throw new Error('reorder failed');
}
