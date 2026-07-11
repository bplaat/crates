/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';

export function FormInput(props: JSX.IntrinsicElements['input']) {
    return <input {...props} class={props.class ? `input ${props.class}` : 'input'} />;
}

export function FormSelect({ children, ...props }: JSX.IntrinsicElements['select']) {
    return (
        <select {...props} class={props.class ? `select ${props.class}` : 'select'}>
            {children}
        </select>
    );
}
