/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import DOMPurify from 'dompurify';
import { marked } from 'marked';
import { useEffect, useRef, useState } from 'preact/hooks';
import TurndownService from 'turndown';
import { t } from '../services/i18n.service.ts';
import {
    CodeBracesIcon,
    CodeTagsIcon,
    FormatBoldIcon,
    FormatItalicIcon,
    FormatListBulletedIcon,
    FormatListNumberedIcon,
    FormatQuoteOpenIcon,
    FormatStrikethroughIcon,
    FormatUnderlineIcon,
    LinkIcon,
    MinusIcon,
} from './icons.tsx';

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
    autoFocus?: boolean;
}

export function RichEditor({ value, onInput, placeholder, class: className, autoFocus }: RichEditorProps) {
    const editorRef = useRef<HTMLDivElement>(null);
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const lastEmittedRef = useRef<string>(value);
    const [isEmpty, setIsEmpty] = useState(!value.trim());
    const [plainMode, setPlainMode] = useState(() => localStorage.getItem('editor-mode') === 'plain');
    const switchedToRichRef = useRef(false);

    useEffect(() => {
        if (!editorRef.current) return;
        editorRef.current.innerHTML = DOMPurify.sanitize(marked.parse(value || '') as string);
        setIsEmpty(!value.trim());
    }, []);

    // Programmatic autofocus on mount - works on every SPA navigation unlike HTML autofocus attribute
    useEffect(() => {
        if (!autoFocus) return;
        const id = requestAnimationFrame(() => {
            const active = document.activeElement;
            if (active && active !== document.body) return;
            if (plainMode) textareaRef.current?.focus();
            else editorRef.current?.focus();
        });
        return () => cancelAnimationFrame(id);
    }, []);

    useEffect(() => {
        if (!editorRef.current) return;
        setIsEmpty(!value.trim());
        if (value !== lastEmittedRef.current) {
            lastEmittedRef.current = value;
            // Don't reset DOM while the editor has focus - user is actively typing
            // and their local content is authoritative. Only update for external changes.
            if (!editorRef.current.contains(document.activeElement)) {
                editorRef.current.innerHTML = DOMPurify.sanitize(marked.parse(value || '') as string);
            }
        }
    }, [value]);

    // When switching back to rich mode, re-render the latest markdown into the editor
    useEffect(() => {
        if (switchedToRichRef.current && editorRef.current) {
            switchedToRichRef.current = false;
            editorRef.current.innerHTML = DOMPurify.sanitize(marked.parse(value || '') as string);
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
                            <FormatBoldIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('italic')} title={t('editor.italic')}>
                            <FormatItalicIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('underline')} title={t('editor.underline')}>
                            <FormatUnderlineIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('strikeThrough')} title={t('editor.strikethrough')}>
                            <FormatStrikethroughIcon class="w-4 h-4" />
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
                            <FormatQuoteOpenIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={() => execCmd('insertUnorderedList')} title={t('editor.ul')}>
                            <FormatListBulletedIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarButton onClick={() => execCmd('insertOrderedList')} title={t('editor.ol')}>
                            <FormatListNumberedIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={handleInlineCode} title={t('editor.code')}>
                            <CodeTagsIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarButton onClick={handleCodeBlock} title={t('editor.code_block')}>
                            <CodeBracesIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={handleLink} title={t('editor.link')}>
                            <LinkIcon class="w-4 h-4" />
                        </ToolbarButton>
                        <ToolbarSep />
                        <ToolbarButton onClick={() => execCmd('insertHorizontalRule')} title={t('editor.hr')}>
                            <MinusIcon class="w-4 h-4" />
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
                    <CodeTagsIcon class="w-3.5 h-3.5" />
                    {plainMode ? t('editor.rich_mode') : t('editor.plain_mode')}
                </button>
            </div>
            {plainMode ? (
                <textarea
                    ref={textareaRef}
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
                <div class="relative overflow-y-auto overflow-x-hidden flex-1">
                    {isEmpty && (
                        <p class="absolute top-5 left-5 text-gray-400 dark:text-gray-600 text-sm pointer-events-none select-none">
                            {placeholder}
                        </p>
                    )}
                    <div
                        ref={editorRef}
                        contentEditable={true}
                        class="rich-editor-content p-5 outline-none min-h-[12rem] text-gray-700 dark:text-gray-300 text-sm break-all"
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
