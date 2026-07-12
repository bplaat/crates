/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { type JSX } from 'preact/jsx-runtime';
import './empty-state.css';
import { cx } from '../utils.ts';

export type EmptyStateProps = JSX.IntrinsicElements['div'] & {
    icon: ComponentChildren;
    message: string;
};

export function EmptyState({ icon, message, class: extraClass, ...props }: EmptyStateProps) {
    return (
        <div {...props} class={cx('empty-state', extraClass)}>
            {icon}
            <p class="empty-state-text">{message}</p>
        </div>
    );
}
