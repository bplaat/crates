/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { Link, useRoute } from 'wouter-preact';
import { IpcContext } from '../app.tsx';
import { $dmxLive, $document, fileName } from '../stage.ts';
import {
    ContentSaveOutlineIcon,
    FilePlusOutlineIcon,
    FolderOpenOutlineIcon,
    MotionPlayOutlineIcon,
    QrcodeIcon,
    ScriptTextOutlineIcon,
    SquareEditOutlineIcon,
} from './icons.tsx';
import { QrModal } from './qrmodal.tsx';
import './header.css';

type UsbStatus =
    | { state: 'connected' }
    | { state: 'disconnected' }
    | {
          state: 'error';
          category: 'access' | 'busy' | 'noDevice' | 'timeout' | 'pipe' | 'unsupported' | 'other';
      };

const USB_ERRORS: Record<Extract<UsbStatus, { state: 'error' }>['category'], { label: string; detail: string }> = {
    access: { label: 'uDMX access denied', detail: 'Check USB permissions and reconnect.' },
    busy: { label: 'uDMX is busy', detail: 'Close other apps using uDMX.' },
    noDevice: { label: 'uDMX disconnected', detail: 'Reconnect uDMX; output resumes automatically.' },
    timeout: { label: 'uDMX timeout', detail: 'The adapter did not respond; retrying.' },
    pipe: { label: 'uDMX transfer stalled', detail: 'The USB transfer stalled; retrying.' },
    unsupported: { label: 'uDMX unsupported', detail: 'Check the installed USB driver.' },
    other: { label: 'uDMX error', detail: 'USB communication failed; retrying.' },
};

/// One status for the whole DMX output: the uDMX connection and whether the stage is live
function outputStatus(usbStatus: UsbStatus, isLive: boolean) {
    if (usbStatus.state === 'error') return { ...USB_ERRORS[usbStatus.category], class: 'is-warning' };
    if (usbStatus.state === 'disconnected') {
        return { label: 'uDMX not connected', detail: 'Connect uDMX; retrying automatically.', class: 'is-danger' };
    }
    if (isLive) return { label: 'DMX live', detail: 'Sending the stage to uDMX.', class: 'is-success' };
    return { label: 'DMX ready', detail: 'uDMX connected, output starts on the stage page.', class: 'is-idle' };
}

function NavLink({ href, children }: { href: string; children: any }) {
    const [isActive] = useRoute(href);
    return (
        <Link href={href} class={`header-tab ${isActive ? 'is-active' : ''}`}>
            {children}
        </Link>
    );
}

export function Header() {
    const ipc = useContext(IpcContext)!;
    const [showQrCode, setShowQrCode] = useState(false);
    const [usbStatus, setUsbStatus] = useState<UsbStatus>({ state: 'disconnected' });
    // File dialogs and window dragging need the native window
    const isApp = ipc.type === 'ipc';

    useEffect(() => {
        const listeners = [
            ipc.on('start', () => ($dmxLive.value = true)),
            ipc.on('stop', () => ($dmxLive.value = false)),
            ipc.on('usbStatusChanged', (data) => {
                setUsbStatus((data as { status: UsbStatus }).status);
            }),
        ];
        ipc.request('getUsbStatus').then(({ status }: any) => setUsbStatus(status));
        return () => listeners.forEach((l) => l.remove());
    }, []);

    const status = outputStatus(usbStatus, $dmxLive.value);
    const path = $document.value?.path;

    return (
        <>
            <header
                class="header"
                // Empty header space acts as the macOS titlebar
                onMouseDown={(event) => {
                    if (!isApp || event.button !== 0 || (event.target as Element).closest('a, button')) return;
                    ipc.send(event.detail === 2 ? 'titlebarDoubleClick' : 'startWindowDrag');
                }}
            >
                <div class="header-start">
                    {path && (
                        <>
                            {isApp && (
                                <>
                                    <button class="icon-button" title="New stage" onClick={() => ipc.send('newStage')}>
                                        <FilePlusOutlineIcon />
                                    </button>
                                    <button
                                        class="icon-button"
                                        title="Open stage..."
                                        onClick={() => ipc.send('openStage')}
                                    >
                                        <FolderOpenOutlineIcon />
                                    </button>
                                    <button
                                        class="icon-button"
                                        title="Save stage as..."
                                        onClick={() => ipc.send('saveStageAs')}
                                    >
                                        <ContentSaveOutlineIcon />
                                    </button>
                                </>
                            )}
                            <span class="header-filename" title={path}>
                                {fileName(path)}
                            </span>
                        </>
                    )}
                </div>

                <nav class="header-tabs">
                    <NavLink href="/">
                        <MotionPlayOutlineIcon />
                        <span class="header-label">Stage</span>
                    </NavLink>
                    <NavLink href="/editor">
                        <SquareEditOutlineIcon />
                        <span class="header-label">Editor</span>
                    </NavLink>
                    <NavLink href="/scripts">
                        <ScriptTextOutlineIcon />
                        <span class="header-label">Scripts</span>
                    </NavLink>
                </nav>

                <div class="header-end">
                    <div
                        class="header-status"
                        role="status"
                        aria-live="polite"
                        aria-atomic="true"
                        title={status.detail}
                    >
                        <span class={`header-dot ${status.class}`} />
                        <span class="header-label">{status.label}</span>
                    </div>

                    <button class="icon-button" title="Show QR-code" onClick={() => setShowQrCode(true)}>
                        <QrcodeIcon />
                    </button>
                </div>
            </header>
            {showQrCode && (
                <QrModal contents={`http://${window.location.host}/`} onClose={() => setShowQrCode(false)} />
            )}
        </>
    );
}
