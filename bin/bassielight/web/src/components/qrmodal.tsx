/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import encodeQR from 'qr';
import { CloseIcon } from './icons.tsx';

export function QrModal({ contents, onClose }: { contents: string; onClose: () => void }) {
    return (
        <div class="modal" onClick={onClose}>
            <button class="button modal-close" onClick={onClose}>
                <CloseIcon />
            </button>

            <div class="qr-code" dangerouslySetInnerHTML={{ __html: encodeQR(contents, 'svg') }} />
            <p class="block">
                <a href={contents} target="_blank">
                    {contents}
                </a>
            </p>
        </div>
    );
}
