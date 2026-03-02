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
        <div class="flex flex-col items-center justify-center mt-24 gap-3 text-gray-400">
            {icon}
            <p class="text-lg">{message}</p>
        </div>
    );
}
