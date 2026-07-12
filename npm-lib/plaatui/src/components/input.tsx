/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './input.css';
import { Icon } from './icons.tsx';
import { cx } from '../utils.ts';

export function FormInput({ class: extraClass, ...props }: JSX.IntrinsicElements['input']) {
    return <input {...props} class={cx('input', extraClass)} />;
}

export function FormSelect({ class: extraClass, children, ...props }: JSX.IntrinsicElements['select']) {
    return (
        <select {...props} class={cx('select', extraClass)}>
            {children}
        </select>
    );
}

export interface SearchInputProps {
    value: string;
    onInput: (value: string) => void;
    onClear: () => void;
    placeholder?: string;
}

export function SearchInput({ value, onInput, onClear, placeholder }: SearchInputProps) {
    return (
        <div class="search">
            <Icon type="magnify" class="search-icon" />
            <input
                type="search"
                value={value}
                onInput={(e) => onInput((e.target as HTMLInputElement).value)}
                placeholder={placeholder}
                class={cx('input', value && 'has-value')}
            />
            {value && (
                <button type="button" onClick={onClear} class="search-clear">
                    <Icon type="close" class="is-sm" />
                </button>
            )}
        </div>
    );
}
