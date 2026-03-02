/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { type JSX } from 'preact/jsx-runtime';

const INPUT_CLASS =
    'w-full px-3 py-2 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm bg-white dark:bg-zinc-700 text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-yellow-400 dark:focus:ring-yellow-500/50 focus:border-transparent';

const BTN_PRIMARY_CLASS =
    'px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer';

const BTN_SECONDARY_CLASS =
    'px-4 py-1.5 rounded-lg text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-zinc-600 transition-colors cursor-pointer';

// Round icon button — page toolbars and action bars (caller must supply a text-color class)
const BTN_ICON_CLASS = 'p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

// Small square icon button — table rows and tight form contexts
const BTN_SMALL_ICON_CLASS =
    'p-1.5 rounded-lg text-gray-400 dark:text-gray-500 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

const BTN_DANGER_CLASS =
    'px-4 py-2 bg-red-500 hover:bg-red-600 dark:bg-red-900/50 dark:hover:bg-red-900/70 dark:text-red-300 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer flex items-center gap-2';

interface FormFieldProps {
    id: string;
    label: string;
    children: ComponentChildren;
}

export function FormField({ id, label, children }: FormFieldProps) {
    return (
        <div class="flex flex-col gap-1">
            <label for={id} class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {label}
            </label>
            {children}
        </div>
    );
}

export function FormInput(props: JSX.IntrinsicElements['input']) {
    return <input {...props} class={INPUT_CLASS} />;
}

export function FormSelect({ children, ...props }: JSX.IntrinsicElements['select']) {
    return (
        <select {...props} class={INPUT_CLASS}>
            {children}
        </select>
    );
}

interface FormMessageProps {
    type: 'success' | 'error';
    message: string | null | undefined | false;
}

export function FormMessage({ type, message }: FormMessageProps) {
    if (!message) return null;
    return (
        <p
            class={
                type === 'success'
                    ? 'text-sm text-green-600 dark:text-green-400'
                    : 'text-sm text-red-500 dark:text-red-400'
            }
        >
            {message}
        </p>
    );
}

export function Button({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_PRIMARY_CLASS} ${extraClass}` : BTN_PRIMARY_CLASS} />;
}

export function SecondaryButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_SECONDARY_CLASS} ${extraClass}` : BTN_SECONDARY_CLASS} />;
}

export function DangerButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_DANGER_CLASS} ${extraClass}` : BTN_DANGER_CLASS} />;
}

export function IconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_ICON_CLASS} ${extraClass}` : BTN_ICON_CLASS} />;
}

export function SmallIconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_SMALL_ICON_CLASS} ${extraClass}` : BTN_SMALL_ICON_CLASS} />;
}

interface SearchInputProps {
    value: string;
    onInput: (value: string) => void;
    onClear: () => void;
    placeholder?: string;
}

export function SearchInput({ value, onInput, onClear, placeholder }: SearchInputProps) {
    return (
        <div class="relative flex items-center">
            <svg
                class="absolute left-2.5 w-4 h-4 text-gray-400 dark:text-gray-500 pointer-events-none shrink-0"
                viewBox="0 0 24 24"
                fill="currentColor"
            >
                <path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
            </svg>
            <input
                type="search"
                value={value}
                onInput={(e) => onInput((e.target as HTMLInputElement).value)}
                placeholder={placeholder}
                class={`${INPUT_CLASS} pl-8 ${value ? 'pr-8' : ''}`}
            />
            {value && (
                <button
                    type="button"
                    onClick={onClear}
                    class="absolute right-2 p-0.5 rounded-full text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors cursor-pointer"
                >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                    </svg>
                </button>
            )}
        </div>
    );
}
