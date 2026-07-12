/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './loading.css';
import { cx } from '../utils.ts';

export type LoadingTextProps = JSX.IntrinsicElements['p'] & {
    // Extra top margin for the first load of a page.
    initial?: boolean;
    // Extra vertical padding, e.g. inside a card.
    padded?: boolean;
};

export function LoadingText({ initial, padded, class: extraClass, ...props }: LoadingTextProps) {
    return <p {...props} class={cx('loading-text', initial && 'is-initial', padded && 'is-padded', extraClass)} />;
}
