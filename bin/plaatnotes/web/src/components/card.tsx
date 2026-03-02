/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';

const CARD_BASE = 'bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm';

interface CardProps {
    class?: string;
    children: ComponentChildren;
}

export function Card({ class: extraClass, children }: CardProps) {
    const cls = extraClass ? `${CARD_BASE} ${extraClass}` : `${CARD_BASE} p-6`;
    return <div class={cls}>{children}</div>;
}
