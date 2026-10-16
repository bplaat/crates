/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { type JSX } from 'preact/jsx-runtime';
import { cx } from '../utils.ts';
import './form.css';

export interface FormFieldProps {
    id: string;
    label: string;
    error?: string | null | false;
    children: ComponentChildren;
}

export function FormField({ id, label, error, children }: FormFieldProps) {
    return (
        <div class="field">
            <label for={id} class="label">
                {label}
            </label>
            {children}
            {error && <p class="help is-danger">{error}</p>}
        </div>
    );
}

export interface FormMessageProps {
    type: 'success' | 'error';
    message: string | null | undefined | false;
}

export function FormMessage({ type, message }: FormMessageProps) {
    if (!message) return null;
    return <p class={type === 'success' ? 'form-message is-success' : 'form-message is-danger'}>{message}</p>;
}

export function Form({ class: extraClass, ...props }: JSX.IntrinsicElements['form']) {
    return <form {...props} class={cx('form', extraClass)} />;
}

export function FormRow({ class: extraClass, ...props }: JSX.IntrinsicElements['div']) {
    return <div {...props} class={cx('field-row', extraClass)} />;
}

export function FormFooter({ class: extraClass, ...props }: JSX.IntrinsicElements['div']) {
    return <div {...props} class={cx('form-footer', extraClass)} />;
}

export type FormActionsProps = JSX.IntrinsicElements['div'] & { flush?: boolean };

export function FormActions({ class: extraClass, flush, ...props }: FormActionsProps) {
    return <div {...props} class={cx('form-actions', flush && 'is-flush', extraClass)} />;
}
