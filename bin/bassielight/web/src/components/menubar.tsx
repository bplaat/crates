/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import './menubar.css';
import { signal } from '@preact/signals';
import { Link, useRoute } from 'wouter-preact';
import { QrModal } from './qrmodal.tsx';
import { CogIcon, MotionPlayOutlineIcon, QrcodeIcon, SquareEditOutlineIcon } from './icons.tsx';
import { IpcContext } from '../app.tsx';

export const $dmxLive = signal(false);

function NavLink({ href, children }: { href: string; children: any }) {
    const [isActive] = useRoute(href);
    return (
        <Link href={href} class={`menubar-item ${isActive ? 'is-active' : ''}`}>
            {children}
        </Link>
    );
}

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
            <div id="menubar" class="menubar">
                <h1 class="menubar-title">BassieLight</h1>

                <NavLink href="/">
                    <MotionPlayOutlineIcon />
                    Stage
                </NavLink>
                <NavLink href="/editor">
                    <SquareEditOutlineIcon />
                    Editor
                </NavLink>
                <NavLink href="/settings">
                    <CogIcon />
                    Settings
                </NavLink>

                <div class="spacer" />

                <div class="menubar-status">
                    <span class={`menubar-dot ${$dmxLive.value ? 'is-success' : 'is-danger'}`} />
                    {$dmxLive.value ? 'DMX is live' : 'DMX is off'}
                </div>

                <button class="menubar-item" onClick={() => setShowQrCode(true)}>
                    <QrcodeIcon />
                    QR-code
                </button>
            </div>
            {showQrCode && (
                <QrModal contents={`http://${window.location.host}/`} onClose={() => setShowQrCode(false)} />
            )}
        </>
    );
}
