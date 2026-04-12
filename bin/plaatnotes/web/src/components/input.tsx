/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';

const INPUT_CLASS =
    'w-full px-3 py-2 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm bg-white dark:bg-zinc-700 text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-yellow-400 dark:focus:ring-yellow-500/50 focus:border-transparent';

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
