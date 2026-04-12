/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import { CloseIcon, MagnifyIcon } from './icons.tsx';

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
            <MagnifyIcon class="absolute left-2.5 w-4 h-4 text-gray-400 dark:text-gray-500 pointer-events-none shrink-0" />
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
                    <CloseIcon class="w-4 h-4" />
                </button>
            )}
        </div>
    );
}
