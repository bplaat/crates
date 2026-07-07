/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { Card } from './card.tsx';
import { RichEditor } from './rich-editor.tsx';

interface NoteEditorCardProps {
    title: string;
    onTitleInput: (title: string) => void;
    titlePlaceholder: string;
    body: string;
    onBodyInput: (body: string) => void;
    bodyPlaceholder: string;
    footer: ComponentChildren;
    onSubmit?: (e: SubmitEvent) => void;
    autoFocus?: boolean;
}

export function NoteEditorCard({
    title,
    onTitleInput,
    titlePlaceholder,
    body,
    onBodyInput,
    bodyPlaceholder,
    footer,
    onSubmit,
    autoFocus,
}: NoteEditorCardProps) {
    const content = (
        <>
            <div class="note-editor-header">
                <input
                    class="note-title-input"
                    type="text"
                    placeholder={titlePlaceholder}
                    value={title}
                    onInput={(e) => onTitleInput((e.target as HTMLInputElement).value)}
                />
            </div>
            <RichEditor value={body} onInput={onBodyInput} placeholder={bodyPlaceholder} autoFocus={autoFocus} />
            <div class="note-editor-footer">{footer}</div>
        </>
    );

    return (
        <Card class="note-editor">
            {onSubmit ? (
                <form onSubmit={onSubmit} class="editor">
                    {content}
                </form>
            ) : (
                content
            )}
        </Card>
    );
}
