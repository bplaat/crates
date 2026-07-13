/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { cx } from '../utils.ts';
import './badge.css';

export interface BadgeProps {
    accent?: boolean;
    class?: string;
    children: ComponentChildren;
}

export function Badge({ accent, class: extraClass, children }: BadgeProps) {
    return <span class={cx('badge', accent && 'is-accent', extraClass)}>{children}</span>;
}
