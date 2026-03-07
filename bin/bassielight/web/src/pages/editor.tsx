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
            <div class="flex-1 flex flex-col">
                <div class="flex-1" />

                <div class="flex flex-wrap gap-2 my-4 items-center justify-center">
                    <button class="flex items-center gap-2 justify-center flex-1 max-w-48 h-12 px-2 border-2 border-transparent rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none">
                        <RectangleOutlineIcon />
                        Add rectangle
                    </button>
                    <button class="flex items-center gap-2 justify-center flex-1 max-w-48 h-12 px-2 border-2 border-transparent rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none">
                        <LightbulbIcon />
                        Add fixture
                    </button>
                </div>
            </div>

            <div class="w-80 bg-zinc-800 p-4 overflow-y-auto" />
        </>
    );
}
