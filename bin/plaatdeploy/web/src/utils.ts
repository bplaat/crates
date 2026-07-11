/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';

export function capitalizeLabel(value: string) {
    return value ? `${value[0].toUpperCase()}${value.slice(1)}` : value;
}

const DEFAULT_TITLE = 'PlaatDeploy';

export function useDocumentTitle(title: string) {
    useEffect(() => {
        document.title = title ? `${title} - ${DEFAULT_TITLE}` : DEFAULT_TITLE;
        return () => {
            document.title = DEFAULT_TITLE;
        };
    }, [title]);
}
