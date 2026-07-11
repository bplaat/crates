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
    return <div class={extraClass ? `card ${extraClass}` : 'card'}>{children}</div>;
}
