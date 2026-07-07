/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import { CloseIcon, MagnifyIcon } from './icons.tsx';

export function FormInput(props: JSX.IntrinsicElements['input']) {
    return <input {...props} class="input" />;
}

export function FormSelect({ children, ...props }: JSX.IntrinsicElements['select']) {
    return (
        <select {...props} class="select">
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
        <div class="search">
            <MagnifyIcon class="search-icon" />
            <input
                type="search"
                value={value}
                onInput={(e) => onInput((e.target as HTMLInputElement).value)}
                placeholder={placeholder}
                class={`input ${value ? 'has-value' : ''}`}
            />
            {value && (
                <button type="button" onClick={onClear} class="search-clear">
                    <CloseIcon class="is-sm" />
                </button>
            )}
        </div>
    );
}
