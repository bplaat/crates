/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { QrModal } from './qrmodal.tsx';
import { CogIcon, MotionPlayOutlineIcon, QrcodeIcon, SquareEditOutlineIcon } from './icons.tsx';
import { Link } from '../router.tsx';

const classes = css`
    .menubar {
        background-color: var(--sidebar-bg);
        display: flex;
        flex-direction: column;
        padding: 1rem;
    }

    :global(body.is-bwebview-macos) .menubar {
        padding-top: 28px;
    }

    .logo {
        font-size: 1.2rem;
        font-weight: bold;
        margin-bottom: 0.75rem;
    }

    .menubar > :global(.button) {
        width: 100%;
        justify-content: start;
        margin-bottom: 0.5rem;
    }
    .menubar > :global(.button):last-child {
        margin-bottom: 0;
    }
`;

export function Menubar() {
    const [showQrCode, setShowQrCode] = useState(false);

    return (
        <>
            <div class={classes['menubar']}>
                <h1 class={classes['logo']}>BassieLight</h1>

                <Link class="button is-text" href="/">
                    <MotionPlayOutlineIcon />
                    Stage
                </Link>
                <Link class="button is-text" href="/editor">
                    <SquareEditOutlineIcon />
                    Editor
                </Link>
                <Link class="button is-text" href="/settings">
                    <CogIcon />
                    Settings
                </Link>

                <div class="flex"></div>

                <div class="button is-text" onClick={() => setShowQrCode(true)}>
                    <QrcodeIcon />
                    QR-code
                </div>
            </div>
            {showQrCode && (
                <QrModal contents={`http://${window.location.host}/`} onClose={() => setShowQrCode(false)} />
            )}
        </>
    );
}
