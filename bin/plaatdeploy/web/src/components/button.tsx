/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';

export function Button({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `button is-primary ${extraClass}` : 'button is-primary'} />;
}

export function SecondaryButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `button is-secondary ${extraClass}` : 'button is-secondary'} />;
}

export function DangerButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `button is-danger ${extraClass}` : 'button is-danger'} />;
}

export function IconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `button is-icon ${extraClass}` : 'button is-icon'} />;
}
