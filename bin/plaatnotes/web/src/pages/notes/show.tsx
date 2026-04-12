/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useLocation, useParams } from 'wouter-preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import { type Note } from '../../../src-gen/api.ts';
import { Navbar } from '../../components/navbar.tsx';
import { IconButton } from '../../components/button.tsx';
import { NoteEditorCard } from '../../components/note-editor-card.tsx';
import { createNote, fetchNote, updateNote } from '../../services/notes.service.ts';
import { formatDate, t } from '../../services/i18n.service.ts';

export function NotesShow() {
    const { note_id } = useParams<{ note_id?: string }>();
    const [, navigate] = useLocation();
    const isNew = !note_id;

    const [isLoading, setIsLoading] = useState(!isNew);
    const [title, setTitle] = useState('');
    const [body, setBody] = useState('');
    const [isPinned, setIsPinned] = useState(false);
    const [isArchived, setIsArchived] = useState(false);
    const [isTrashed, setIsTrashed] = useState(false);
    const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved'>('idle');

    const noteRef = useRef<Note | null>(null);
    const saveTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
    const savedTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const pendingData = useRef<{ title: string; body: string; isPinned: boolean } | null>(null);

    useEffect(() => {
        if (isNew) {
            document.title = `PlaatNotes - ${t('page.create')}`;
            return;
        }
        void (async () => {
            document.title = `PlaatNotes - ${t('page.note_loading')}`;
            const loaded = await fetchNote(note_id!);
            if (!loaded) return;
            noteRef.current = loaded;
            setTitle(loaded.title || '');
            setBody(loaded.body);
            setIsPinned(loaded.isPinned);
            setIsArchived(loaded.isArchived);
            setIsTrashed(loaded.isTrashed);
            setIsLoading(false);
            document.title = `PlaatNotes - ${loaded.title || loaded.body.slice(0, 40)}`;
        })();
    }, [note_id]);

    // Flush pending save on unmount
    useEffect(() => {
        return () => {
            if (saveTimeout.current) {
                clearTimeout(saveTimeout.current);
                const data = pendingData.current;
                if (data) {
                    if (noteRef.current) {
                        updateNote(noteRef.current, {});
                    } else if (data.body.trim()) {
                        createNote({ body: data.body, title: data.title || undefined, isPinned: data.isPinned });
                    }
                }
            }
            if (savedTimer.current) clearTimeout(savedTimer.current);
        };
    }, []);

    function scheduleSave(newTitle: string, newBody: string, newIsPinned: boolean) {
        setSaveStatus('saving');
        pendingData.current = { title: newTitle, body: newBody, isPinned: newIsPinned };
        if (saveTimeout.current) clearTimeout(saveTimeout.current);
        saveTimeout.current = setTimeout(async () => {
            saveTimeout.current = null;
            pendingData.current = null;
            if (noteRef.current) {
                const saved = await updateNote(noteRef.current, {});
                noteRef.current = { ...noteRef.current, updatedAt: saved.updatedAt };
                setSaveStatus('saved');
            } else if (newBody.trim()) {
                const created = await createNote({
                    body: newBody,
                    title: newTitle || undefined,
                    isPinned: newIsPinned,
                });
                if (created) {
                    noteRef.current = created;
                    window.history.replaceState(null, '', `/notes/${created.id}`);
                    setSaveStatus('saved');
                }
            } else {
                setSaveStatus('idle');
                return;
            }
            if (savedTimer.current) clearTimeout(savedTimer.current);
            savedTimer.current = setTimeout(() => setSaveStatus('idle'), 2000);
        }, 600);
    }

    function handleTitleInput(newTitle: string) {
        setTitle(newTitle);
        if (noteRef.current) noteRef.current = { ...noteRef.current, title: newTitle };
        scheduleSave(newTitle, body, isPinned);
    }

    function handleBodyInput(newBody: string) {
        setBody(newBody);
        if (noteRef.current) noteRef.current = { ...noteRef.current, body: newBody };
        scheduleSave(title, newBody, isPinned);
    }

    async function handlePin() {
        const newIsPinned = !isPinned;
        setIsPinned(newIsPinned);
        if (noteRef.current) {
            noteRef.current = { ...noteRef.current, isPinned: newIsPinned };
            const saved = await updateNote(noteRef.current, {});
            noteRef.current = { ...noteRef.current, updatedAt: saved.updatedAt };
        }
    }

    async function handleArchive() {
        if (!noteRef.current) return;
        noteRef.current = { ...noteRef.current, isArchived: !isArchived };
        await updateNote(noteRef.current, {});
        navigate(isArchived ? '/' : '/archive');
    }

    async function handleTrash() {
        if (!noteRef.current) return;
        noteRef.current = { ...noteRef.current, isTrashed: !isTrashed };
        await updateNote(noteRef.current, {});
        navigate(isTrashed ? '/' : '/trash');
    }

    if (isLoading) {
        return (
            <div class="min-h-screen bg-gray-50 dark:bg-zinc-900">
                <Navbar />
                <p class="text-center text-gray-400 dark:text-gray-500 mt-24">{t('app.loading')}</p>
            </div>
        );
    }

    return (
        <div class="h-screen flex flex-col bg-gray-50 dark:bg-zinc-900">
            <Navbar />
            <main class="flex-1 flex flex-col min-h-0 max-w-2xl w-full mx-auto px-4 py-8">
                <div class="flex items-center gap-3 mb-6">
                    <IconButton
                        onClick={() => navigate(isArchived ? '/archive' : isTrashed ? '/trash' : '/')}
                        class="text-gray-500 dark:text-gray-400"
                        title={t('notes_show.back')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
                        </svg>
                    </IconButton>
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">
                        {isNew ? t('notes_create.heading') : t('notes_show.heading')}
                    </h1>
                    <div class="flex-1" />
                    <IconButton
                        onClick={handlePin}
                        class={isPinned ? 'text-yellow-500 dark:text-yellow-400' : 'text-gray-400 dark:text-gray-500'}
                        title={isPinned ? t('note.unpin') : t('note.pin')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
                        </svg>
                    </IconButton>
                    {!isNew && (
                        <>
                            <IconButton
                                onClick={handleArchive}
                                class={
                                    isArchived
                                        ? 'text-yellow-500 dark:text-yellow-400'
                                        : 'text-gray-400 dark:text-gray-500'
                                }
                                title={isArchived ? t('note.unarchive') : t('note.archive')}
                            >
                                {isArchived ? (
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20.55 5.22l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.15.55L3.46 5.22C3.17 5.57 3 6.01 3 6.5V19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.49-.17-.93-.45-1.28zM12 9.5l5.5 5.5H14v2h-4v-2H6.5L12 9.5zM5.12 5l.82-1h12l.93 1H5.12z" />
                                    </svg>
                                ) : (
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                                    </svg>
                                )}
                            </IconButton>
                            <IconButton
                                onClick={handleTrash}
                                class={
                                    isTrashed ? 'text-red-500 dark:text-red-400' : 'text-gray-400 dark:text-gray-500'
                                }
                                title={isTrashed ? t('note.restore') : t('note.trash')}
                            >
                                {isTrashed ? (
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M13 3a9 9 0 0 0-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0 0 13 21a9 9 0 0 0 0-18z" />
                                    </svg>
                                ) : (
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                                    </svg>
                                )}
                            </IconButton>
                        </>
                    )}
                </div>

                <NoteEditorCard
                    title={title}
                    onTitleInput={handleTitleInput}
                    titlePlaceholder={isNew ? t('notes_create.title_placeholder') : t('notes_show.title_placeholder')}
                    body={body}
                    onBodyInput={handleBodyInput}
                    bodyPlaceholder={isNew ? t('notes_create.body_placeholder') : t('notes_show.body_placeholder')}
                    autoFocus={isNew}
                    footer={
                        <p class="text-xs text-gray-400 dark:text-gray-500">
                            {saveStatus === 'saving' && (
                                <span class="text-yellow-500 dark:text-yellow-400">{t('notes_show.saving')}</span>
                            )}
                            {saveStatus === 'saved' && (
                                <span class="text-green-500 dark:text-green-400">{t('notes_show.saved')}</span>
                            )}
                            {saveStatus === 'idle' &&
                                noteRef.current &&
                                t('notes_show.last_updated', formatDate(noteRef.current.updatedAt))}
                        </p>
                    }
                />
            </main>
        </div>
    );
}
