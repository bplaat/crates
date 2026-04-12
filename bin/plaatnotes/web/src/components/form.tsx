/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { type JSX } from 'preact/jsx-runtime';

interface FormFieldProps {
    id: string;
    label: string;
    error?: string | null | false;
    children: ComponentChildren;
}

export function FormField({ id, label, error, children }: FormFieldProps) {
    return (
        <div class="flex flex-col gap-1">
            <label for={id} class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {label}
            </label>
            {children}
            {error && <p class="text-xs text-red-500 dark:text-red-400">{error}</p>}
        </div>
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

const FORM_ACTIONS_CLASS = 'flex justify-end gap-2 pt-1';

export function FormActions({ class: extraClass, ...props }: JSX.IntrinsicElements['div']) {
    return <div {...props} class={extraClass ? `${FORM_ACTIONS_CLASS} ${extraClass}` : FORM_ACTIONS_CLASS} />;
}
