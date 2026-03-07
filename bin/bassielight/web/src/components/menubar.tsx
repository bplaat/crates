/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { signal } from '@preact/signals';
import { Link, useRoute } from 'wouter-preact';
import { QrModal } from './qrmodal.tsx';
import { CogIcon, MotionPlayOutlineIcon, QrcodeIcon, SquareEditOutlineIcon } from './icons.tsx';
import { IpcContext } from '../app.tsx';

export const $dmxLive = signal(false);

const NAV_BASE =
    'flex items-center gap-2 w-full px-2 h-12 rounded-lg font-medium text-zinc-200 cursor-pointer transition-all border-2 border-transparent hover:bg-zinc-700 hover:border-blue-500 mb-2 no-underline! outline-none';

function NavLink({ href, children }: { href: string; children: any }) {
    const [isActive] = useRoute(href);
    return (
        <Link href={href} class={`${NAV_BASE} ${isActive ? 'border-blue-500!' : ''}`}>
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
            <div id="menubar" class="bg-zinc-800 flex flex-col p-4">
                <h1 class="text-xl font-bold mb-3">BassieLight</h1>

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

                <div class="flex-1" />

                <div class="flex items-center gap-2 mb-2">
                    <span class={`w-2 h-2 rounded-full ${$dmxLive.value ? 'bg-green-500' : 'bg-red-500'}`} />
                    {$dmxLive.value ? 'DMX is live' : 'DMX is off'}
                </div>

                <button class={`${NAV_BASE} mb-0`} onClick={() => setShowQrCode(true)}>
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
