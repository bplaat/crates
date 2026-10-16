/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    ArrowLeftIcon,
    DeleteIcon,
    IconButton,
    LoadingText,
    PackageDownIcon,
    PackageUpIcon,
    PinIcon,
    RestoreIcon,
} from 'plaatui';
import { useEffect, useRef, useState } from 'preact/hooks';
import { useLocation, useParams } from 'wouter-preact';
import { type Note } from '../../../src-gen/api.ts';
import { PlaatNotesNavbar } from '../../components/navbar.tsx';
import { NoteEditorCard } from '../../components/note-editor-card.tsx';
import { formatDate, t } from '../../services/i18n.service.ts';
import { createNote, fetchNote, updateNote } from '../../services/notes.service.ts';
import './show.css';

function hasMeaningfulBody(body: string): boolean {
    return body.replace(/<br\s*\/?>/gi, '').trim().length > 0;
}

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
                    } else if (hasMeaningfulBody(data.body)) {
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
            } else if (hasMeaningfulBody(newBody)) {
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
            <div class="layout">
                <PlaatNotesNavbar />
                <LoadingText initial>{t('app.loading')}</LoadingText>
            </div>
        );
    }

    return (
        <div class="note-show">
            <PlaatNotesNavbar />
            <main class="note-show-main">
                <div class="note-show-header">
                    <IconButton
                        onClick={() => navigate(isArchived ? '/archive' : isTrashed ? '/trash' : '/')}
                        class="has-text-muted"
                        title={t('notes_show.back')}
                    >
                        <ArrowLeftIcon class="is-md" />
                    </IconButton>
                    <h1 class="note-show-title">{isNew ? t('notes_create.heading') : t('notes_show.heading')}</h1>
                    <div class="spacer" />
                    <IconButton
                        onClick={handlePin}
                        class={isPinned ? 'is-active' : 'has-text-muted'}
                        title={isPinned ? t('note.unpin') : t('note.pin')}
                    >
                        <PinIcon class="is-md" />
                    </IconButton>
                    {!isNew && (
                        <>
                            <IconButton
                                onClick={handleArchive}
                                class={isArchived ? 'is-active' : 'has-text-muted'}
                                title={isArchived ? t('note.unarchive') : t('note.archive')}
                            >
                                {isArchived ? <PackageUpIcon class="is-md" /> : <PackageDownIcon class="is-md" />}
                            </IconButton>
                            <IconButton
                                onClick={handleTrash}
                                class={isTrashed ? 'is-active-danger' : 'has-text-muted'}
                                title={isTrashed ? t('note.restore') : t('note.trash')}
                            >
                                {isTrashed ? <RestoreIcon class="is-md" /> : <DeleteIcon class="is-md" />}
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
                        <p class="save-status">
                            {saveStatus === 'saving' && <span class="is-saving">{t('notes_show.saving')}</span>}
                            {saveStatus === 'saved' && <span class="is-saved">{t('notes_show.saved')}</span>}
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
