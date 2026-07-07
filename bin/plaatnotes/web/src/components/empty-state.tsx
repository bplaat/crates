/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';

interface EmptyStateProps {
    icon: JSX.Element;
    message: string;
}

export function EmptyState({ icon, message }: EmptyStateProps) {
    return (
        <div class="empty-state">
            {icon}
            <p class="empty-state-text">{message}</p>
        </div>
    );
}
