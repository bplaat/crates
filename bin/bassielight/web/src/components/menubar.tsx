/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { signal } from '@preact/signals';
import { QrModal } from './qrmodal.tsx';
import { CogIcon, MotionPlayOutlineIcon, QrcodeIcon, SquareEditOutlineIcon } from './icons.tsx';
import { Link } from '../router.tsx';
import { IpcContext } from '../app.tsx';

export const $dmxLive = signal(false);

const classes = css`
    /* Menubar */
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

    /* DMX state */
    .dmx-state {
        display: flex;
        align-items: center;
        margin-bottom: 0.5rem;
    }
    .dmx-state.is-live:before,
    .dmx-state.is-off:before {
        content: '';
        display: block;
        width: 0.5rem;
        height: 0.5rem;
        border-radius: 50%;
        margin-right: 0.5rem;
    }
    .dmx-state.is-live:before {
        background-color: var(--success-color);
    }
    .dmx-state.is-off:before {
        background-color: var(--danger-color);
    }
`;

export function Menubar() {
    const ipc = useContext(IpcContext)!;
    const [showQrCode, setShowQrCode] = useState(false);

    useEffect(() => {
        const listeners = [
            ipc.on('start', () => ($dmxLive.value = true)),
            ipc.on('stop', () => ($dmxLive.value = false)),
        ];
        return () => listeners.forEach((l) => l.remove());
    }, []);

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

                <div class={`${classes['dmx-state']} ${$dmxLive.value ? classes['is-live'] : classes['is-off']}`}>
                    {$dmxLive.value ? 'DMX is live' : 'DMX is off'}
                </div>

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
