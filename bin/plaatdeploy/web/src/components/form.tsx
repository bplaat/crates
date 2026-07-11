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
    help?: ComponentChildren;
    error?: string | null | false;
    children: ComponentChildren;
}

export function FormField({ id, label, help, error, children }: FormFieldProps) {
    return (
        <div class="field">
            <label for={id} class="label">
                {label}
            </label>
            {children}
            {error ? <p class="help is-danger">{error}</p> : help ? <p class="help">{help}</p> : null}
        </div>
    );
}

interface FormMessageProps {
    type: 'success' | 'error';
    message: string | null | undefined | false;
}

export function FormMessage({ type, message }: FormMessageProps) {
    if (!message) return null;
    return <p class={type === 'success' ? 'form-message is-success' : 'form-message is-danger'}>{message}</p>;
}

export function FormActions({ class: extraClass, ...props }: JSX.IntrinsicElements['div']) {
    return <div {...props} class={extraClass ? `form-actions ${extraClass}` : 'form-actions'} />;
}
