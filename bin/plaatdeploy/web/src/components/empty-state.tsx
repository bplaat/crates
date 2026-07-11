/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';

interface EmptyStateProps {
    icon?: ComponentChildren;
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
