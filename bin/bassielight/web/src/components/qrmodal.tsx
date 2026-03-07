/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import encodeQR from 'qr';
import { CloseIcon } from './icons.tsx';

export function QrModal({ contents, onClose }: { contents: string; onClose: () => void }) {
    return (
        <div class="fixed inset-0 z-[999] bg-black/80 flex flex-col items-center justify-center" onClick={onClose}>
            <button
                class="absolute top-4 right-4 p-2 border-2 border-transparent rounded-lg bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:scale-95 outline-none"
                onClick={onClose}
            >
                <CloseIcon />
            </button>

            <div
                class="w-[60vw] max-w-[60vh] bg-white"
                dangerouslySetInnerHTML={{ __html: encodeQR(contents, 'svg') }}
            />
            <p class="mt-4">
                <a href={contents} target="_blank">
                    {contents}
                </a>
            </p>
        </div>
    );
}
