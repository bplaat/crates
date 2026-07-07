/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';

interface CardProps {
    class?: string;
    children: ComponentChildren;
}

export function Card({ class: extraClass, children }: CardProps) {
    const cls = extraClass ? `card ${extraClass}` : 'card is-padded';
    return <div class={cls}>{children}</div>;
}
