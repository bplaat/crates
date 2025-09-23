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
                <div class="flex"></div>

                <div class="buttons is-centered">
                    <button class="button is-text is-large">
                        <RectangleOutlineIcon />
                        Add rectangle
                    </button>
                    <button class="button is-text is-large">
                        <LightbulbIcon />
                        Add fixture
                    </button>
                </div>
            </div>

            <div class="sidebar"></div>
        </>
    );
}
