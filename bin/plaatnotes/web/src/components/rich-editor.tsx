/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { marked } from 'marked';
import { useEffect, useRef, useState } from 'preact/hooks';
import TurndownService from 'turndown';
import { t } from '../services/i18n.service.ts';

const td = new TurndownService({ headingStyle: 'atx', bulletListMarker: '-' });
td.addRule('underline', {
    filter: ['u'],
    replacement: (content) => `<u>${content}</u>`,
});
td.addRule('strikethrough', {
    // @ts-ignore
    filter: ['s', 'strike', 'del'],
    replacement: (content) => `~~${content}~~`,
});
// Preserve <br> as HTML so empty lines round-trip correctly
td.addRule('br', {
    filter: 'br',
    replacement: () => '<br>',
});
// Empty paragraphs (blank lines in editor) → <br> placeholder in markdown
td.addRule('empty-paragraph', {
    filter: (node) => node.nodeName === 'P' && !node.textContent?.trim(),
    replacement: () => '<br>',
});
// Use --- for horizontal rules
td.addRule('hr', {
    filter: 'hr',
    replacement: () => '\n\n---\n\n',
});

interface ToolbarButtonProps {
    onClick: () => void;
    title: string;
    children: preact.ComponentChildren;
}

function ToolbarButton({ onClick, title, children }: ToolbarButtonProps) {
    return (
        <button
            type="button"
            onMouseDown={(e) => {
                e.preventDefault();
                onClick();
            }}
            title={title}
            class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-zinc-600 text-gray-600 dark:text-gray-300 transition-colors cursor-pointer"
        >
            {children}
        </button>
    );
}

function ToolbarSep() {
    return <div class="w-px h-5 bg-gray-200 dark:bg-zinc-600 mx-1 self-center" />;
}

interface RichEditorProps {
    value: string;
    onInput: (markdown: string) => void;
    placeholder?: string;
    class?: string;
}

export function RichEditor({ value, onInput, placeholder, class: className }: RichEditorProps) {
    const editorRef = useRef<HTMLDivElement>(null);
    const lastEmittedRef = useRef<string>(value);
    const [isEmpty, setIsEmpty] = useState(!value.trim());
    const [plainMode, setPlainMode] = useState(() => localStorage.getItem('editor-mode') === 'plain');
    const switchedToRichRef = useRef(false);

    useEffect(() => {
        if (!editorRef.current) return;
        editorRef.current.innerHTML = marked.parse(value || '') as string;
        setIsEmpty(!value.trim());
    }, []);

    useEffect(() => {
        if (!editorRef.current) return;
        setIsEmpty(!value.trim());
        if (value !== lastEmittedRef.current) {
            lastEmittedRef.current = value;
            // Don't reset DOM while the editor has focus — user is actively typing
            // and their local content is authoritative. Only update for external changes.
            if (!editorRef.current.contains(document.activeElement)) {
                editorRef.current.innerHTML = marked.parse(value || '') as string;
            }
        }
    }, [value]);

    // When switching back to rich mode, re-render the latest markdown into the editor
    useEffect(() => {
        if (switchedToRichRef.current && editorRef.current) {
            switchedToRichRef.current = false;
            editorRef.current.innerHTML = marked.parse(value || '') as string;
            lastEmittedRef.current = value;
        }
    }, [plainMode]);

    function emit() {
        if (!editorRef.current) return;
        const markdown = td.turndown(editorRef.current.innerHTML);
        lastEmittedRef.current = markdown;
        onInput(markdown);
    }

    function execCmd(cmd: string, val?: string) {
        document.execCommand(cmd, false, val);
        emit();
    }

    function handleLink() {
        const url = prompt(t('editor.link_prompt'));
        if (url) execCmd('createLink', url);
    }

    function handleInlineCode() {
        const sel = window.getSelection();
        const text = sel?.toString() || 'code';
        document.execCommand('insertHTML', false, `<code>${text}</code>`);
        emit();
    }

    function handleCodeBlock() {
        const sel = window.getSelection();
        const text = sel?.toString() || 'code';
        document.execCommand('insertHTML', false, `<pre><code>${text}</code></pre>`);
        emit();
    }

    function togglePlainMode() {
        if (plainMode) {
            switchedToRichRef.current = true;
        }
        const next = !plainMode;
        localStorage.setItem('editor-mode', next ? 'plain' : 'rich');
        setPlainMode(next);
    }

    return (
        <div class={`flex flex-col ${className || ''}`}>
            <div class="flex items-center flex-wrap gap-0.5 px-2 h-10 shrink-0 border-b border-gray-100 dark:border-zinc-700 bg-gray-50 dark:bg-zinc-800/50">
                {!plainMode && (
                    <>
                        <ToolbarButton onClick={() => execCmd('bold')} title={t('editor.bold')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M15.6 10.79c.97-.67 1.65-1.77 1.65-2.79 0-2.26-1.75-4-4-4H7v14h7.04c2.09 0 3.71-1.7 3.71-3.79 0-1.52-.86-2.82-2.15-3.42zM10 6.5h3c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5h-3v-3zm3.5 9H10v-3h3.5c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('italic')} title={t('editor.italic')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M10 4v3h2.21l-3.42 8H6v3h8v-3h-2.21l3.42-8H18V4h-8z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('underline')} title={t('editor.underline')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 17c3.31 0 6-2.69 6-6V3h-2.5v8c0 1.93-1.57 3.5-3.5 3.5S8.5 12.93 8.5 11V3H6v8c0 3.31 2.69 6 6 6zm-7 2v2h14v-2H5z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('strikeThrough')} title={t('editor.strikethrough')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M10 19h4v-3h-4v3zM5 4v3h5v3h4V7h5V4H5zM3 14h18v-2H3v2z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={() => execCmd('formatBlock', 'h1')} title={t('editor.h1')}>
                            <span class="text-xs font-bold px-0.5 leading-none">H1</span>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('formatBlock', 'h2')} title={t('editor.h2')}>
                            <span class="text-xs font-bold px-0.5 leading-none">H2</span>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('formatBlock', 'h3')} title={t('editor.h3')}>
                            <span class="text-xs font-bold px-0.5 leading-none">H3</span>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('formatBlock', 'p')} title={t('editor.p')}>
                            <span class="text-xs px-0.5 leading-none">¶</span>
                        </ToolbarButton>
                        <ToolbarButton
                            onClick={() => execCmd('formatBlock', 'blockquote')}
                            title={t('editor.blockquote')}
                        >
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M6 17h3l2-4V7H5v6h3zm8 0h3l2-4V7h-6v6h3z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={() => execCmd('insertUnorderedList')} title={t('editor.ul')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M4 10.5c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5 1.5-.67 1.5-1.5-.67-1.5-1.5-1.5zm0-6c-.83 0-1.5.67-1.5 1.5S3.17 7.5 4 7.5 5.5 6.83 5.5 6 4.83 4.5 4 4.5zm0 12c-.83 0-1.5.68-1.5 1.5s.68 1.5 1.5 1.5 1.5-.68 1.5-1.5-.67-1.5-1.5-1.5zM7 19h14v-2H7v2zm0-6h14v-2H7v2zm0-8v2h14V5H7z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('insertOrderedList')} title={t('editor.ol')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M2 17h2v.5H3v1h1v.5H2v1h3v-4H2v1zm1-9h1V4H2v1h1v3zm-1 3h1.8L2 13.1v.9h3v-1H3.2L5 10.9V10H2v1zm5-6v2h14V5H7zm0 14h14v-2H7v2zm0-6h14v-2H7v2z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={handleInlineCode} title={t('editor.code')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarButton onClick={handleCodeBlock} title={t('editor.code_block')}>
                            <svg
                                class="w-4 h-4"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                            >
                                <path d="M5.59 3.41L7 4.82L3.82 8L7 11.18L5.59 12.6L1 8L5.59 3.41M11.41 3.41L16 8L11.41 12.6L10 11.18L13.18 8L10 4.82L11.41 3.41M22 6V18C22 19.11 21.11 20 20 20H4C2.9 20 2 19.11 2 18V14H4V18H20V6H17.03V4H20C21.11 4 22 4.89 22 6Z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={handleLink} title={t('editor.link')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z" />
                            </svg>
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={() => execCmd('insertHorizontalRule')} title={t('editor.hr')}>
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M19 13H5v-2h14v2z" />
                            </svg>
                        </ToolbarButton>
                    </>
                )}
                <div class="flex-1" />
                <button
                    type="button"
                    onMouseDown={(e) => {
                        e.preventDefault();
                        togglePlainMode();
                    }}
                    title={plainMode ? t('editor.rich_mode') : t('editor.plain_mode')}
                    class="flex items-center gap-1.5 px-2 py-1 rounded text-xs text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-zinc-600 transition-colors cursor-pointer"
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z" />
                    </svg>
                    {plainMode ? t('editor.rich_mode') : t('editor.plain_mode')}
                </button>
            </div>
            {plainMode ? (
                <textarea
                    class="flex-1 p-5 outline-none min-h-[12rem] font-mono text-sm text-gray-700 dark:text-gray-300 bg-transparent resize-none placeholder-gray-400 dark:placeholder-gray-600"
                    placeholder={placeholder}
                    value={value}
                    onKeyDown={(e) => {
                        if (e.key === 'Tab') {
                            e.preventDefault();
                            const el = e.target as HTMLTextAreaElement;
                            const start = el.selectionStart;
                            const end = el.selectionEnd;
                            const markdown = el.value.substring(0, start) + '    ' + el.value.substring(end);
                            el.value = markdown;
                            el.selectionStart = el.selectionEnd = start + 4;
                            lastEmittedRef.current = markdown;
                            onInput(markdown);
                        }
                    }}
                    onInput={(e) => {
                        const markdown = (e.target as HTMLTextAreaElement).value;
                        lastEmittedRef.current = markdown;
                        onInput(markdown);
                    }}
                />
            ) : (
                <div class="relative overflow-y-auto flex-1">
                    {isEmpty && (
                        <p class="absolute top-5 left-5 text-gray-400 dark:text-gray-600 text-sm pointer-events-none select-none">
                            {placeholder}
                        </p>
                    )}
                    <div
                        ref={editorRef}
                        contentEditable={true}
                        class="rich-editor-content p-5 outline-none min-h-[12rem] text-gray-700 dark:text-gray-300 text-sm"
                        onKeyDown={(e) => {
                            if (e.key === 'Tab') {
                                e.preventDefault();
                                document.execCommand('insertText', false, '    ');
                            }
                        }}
                        onInput={emit}
                    />
                </div>
            )}
        </div>
    );
}
