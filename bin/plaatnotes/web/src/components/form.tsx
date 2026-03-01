/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { type JSX } from 'preact/jsx-runtime';

const BTN_ICON_CLASS =
    'p-1.5 rounded-lg text-gray-400 dark:text-gray-500 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

const INPUT_CLASS =
    'w-full px-3 py-2 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm bg-white dark:bg-zinc-700 text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-yellow-400 dark:focus:ring-yellow-500/50 focus:border-transparent';

const BTN_PRIMARY_CLASS =
    'px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer';

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

export function Button({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_PRIMARY_CLASS} ${extraClass}` : BTN_PRIMARY_CLASS} />;
}

export function IconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_ICON_CLASS} ${extraClass}` : BTN_ICON_CLASS} />;
}
