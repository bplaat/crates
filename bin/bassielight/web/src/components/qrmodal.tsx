/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import encodeQR from 'qr';
import { CloseIcon } from './icons.tsx';

const classes = css`
    .qr-modal {
        position: fixed;
        top: 0px;
        left: 0px;
        width: 100vw;
        height: 100vh;
        background-color: rgba(0, 0, 0, 0.8);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    .qr-code {
        width: 60vw;
        max-width: 60vh;
        background-color: #fff;
    }

    .close-button {
        position: absolute;
        top: 1rem;
        right: 1rem;
    }
`;

export function QrModal({ contents, onClose }: { contents: string; onClose: () => void }) {
    return (
        <div class={classes['qr-modal']} onClick={onClose}>
            <button class={`button is-icon ${classes['close-button']}`} onClick={onClose}>
                <CloseIcon />
            </button>

            <div class={classes['qr-code']} dangerouslySetInnerHTML={{ __html: encodeQR(contents, 'svg') }} />
            <p>
                <a href={contents} target="_blank">
                    {contents}
                </a>
            </p>
        </div>
    );
}
