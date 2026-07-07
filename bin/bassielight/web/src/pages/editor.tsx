/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';
import { LightbulbIcon, RectangleOutlineIcon } from '../components/icons.tsx';

export function EditorPage() {
    useEffect(() => {
        document.title = 'BassieLight - Editor';
    }, []);

    return (
        <>
            <div class="main">
                <div class="spacer" />

                <div class="buttons is-centered">
                    <button class="button is-expanded">
                        <RectangleOutlineIcon />
                        Add rectangle
                    </button>
                    <button class="button is-expanded">
                        <LightbulbIcon />
                        Add fixture
                    </button>
                </div>
            </div>

            <div class="sidebar" />
        </>
    );
}
