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
            <div class="px-5 pt-5 pb-2">
                <input
                    class="text-xl font-medium text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-600 bg-transparent outline-none w-full"
                    type="text"
                    placeholder={titlePlaceholder}
                    value={title}
                    onInput={(e) => onTitleInput((e.target as HTMLInputElement).value)}
                />
            </div>
            <RichEditor
                class="flex-1 min-h-0"
                value={body}
                onInput={onBodyInput}
                placeholder={bodyPlaceholder}
                autoFocus={autoFocus}
            />
            <div class="border-t border-gray-100 dark:border-zinc-700 px-5 py-3 bg-gray-50 dark:bg-zinc-700/50">
                {footer}
            </div>
        </>
    );

    return (
        <Card class="flex-1 flex flex-col min-h-0 overflow-hidden">
            {onSubmit ? (
                <form onSubmit={onSubmit} class="flex-1 flex flex-col min-h-0">
                    {content}
                </form>
            ) : (
                content
            )}
        </Card>
    );
}
