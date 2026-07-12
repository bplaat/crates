/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import '../components/note-grid.css';
import '../components/toolbar.css';
import { type Note } from '../../src-gen/api.ts';
import { ConfirmDialog } from 'plaatui';
import { EmptyState } from 'plaatui';
import { AppLayout } from '../components/app-layout.tsx';
import { NoteCard } from '../components/note-card.tsx';
import { useInfiniteScroll } from '../hooks/use-infinite-scroll.ts';
import { deleteNote, listTrashedNotes, updateNote, clearTrashedNotes } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { DangerTextButton, Icon, LoadingText } from 'plaatui';
import { useSearchQuery } from '../hooks/use-search-query.ts';
import './trash.css';

type ConfirmAction = { kind: 'delete'; note: Note } | { kind: 'empty' } | null;

export function TrashPage() {
    const query = useSearchQuery();
    const {
        items: notes,
        loading,
        hasMore,
        sentinelRef,
        setItems: setNotes,
    } = useInfiniteScroll(listTrashedNotes, query);
    const [confirmAction, setConfirmAction] = useState<ConfirmAction>(null);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.trash')}`;
    }, []);

    async function handleRestore(note: Note) {
        await updateNote(note, { isTrashed: false });
        setNotes((ns) => ns.filter((n) => n.id !== note.id));
    }

    function handleDeleteForever(note: Note) {
        setConfirmAction({ kind: 'delete', note });
    }

    function handleEmptyTrash() {
        setConfirmAction({ kind: 'empty' });
    }

    async function doConfirm() {
        if (confirmAction?.kind === 'delete') {
            await deleteNote(confirmAction.note.id);
            setNotes((ns) => ns.filter((n) => n.id !== confirmAction.note.id));
        } else if (confirmAction?.kind === 'empty') {
            await clearTrashedNotes();
            setNotes([]);
        }
        setConfirmAction(null);
    }

    return (
        <>
            <AppLayout showSearch>
                <div class="page">
                    <div class="toolbar is-relative">
                        <h1 class="section-label is-spaced">{t('trash.heading')}</h1>
                        {notes.length > 0 && (
                            <DangerTextButton onClick={handleEmptyTrash} class="empty-trash-btn">
                                <Icon type="delete-outline" class="is-sm" />
                                {t('trash.empty_btn')}
                            </DangerTextButton>
                        )}
                    </div>

                    {!loading && notes.length > 0 && <p class="hint-text">{t('trash.hint')}</p>}

                    {loading && notes.length === 0 && <LoadingText initial>{t('trash.loading')}</LoadingText>}

                    {!loading && notes.length === 0 && (
                        <EmptyState
                            icon={<Icon type="delete-outline" class="is-huge" />}
                            message={query ? t('trash.empty_search') : t('trash.empty')}
                        />
                    )}

                    {notes.length > 0 && (
                        <div class="note-grid">
                            {notes.map((note) => (
                                <NoteCard
                                    key={note.id}
                                    note={note}
                                    onRestore={handleRestore}
                                    onDeleteForever={handleDeleteForever}
                                />
                            ))}
                        </div>
                    )}

                    {hasMore && <div ref={sentinelRef} class="sentinel" />}
                    {loading && notes.length > 0 && <LoadingText>{t('trash.loading')}</LoadingText>}
                </div>
            </AppLayout>

            {confirmAction && (
                <ConfirmDialog
                    title={confirmAction.kind === 'delete' ? t('note.delete_forever') : t('trash.empty_btn')}
                    message={confirmAction.kind === 'delete' ? t('trash.confirm_delete') : t('trash.confirm_empty')}
                    confirmLabel={confirmAction.kind === 'delete' ? t('note.delete_forever') : t('trash.empty_btn')}
                    cancelLabel={t('dialog.cancel')}
                    onConfirm={doConfirm}
                    onClose={() => setConfirmAction(null)}
                />
            )}
        </>
    );
}
