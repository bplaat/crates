/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect } from 'preact/hooks';
import encodeQR from 'qr';
import Ipc from './ipc.js';

const ipc = new Ipc();

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 750, 1000];
const MODES = [
    {
        type: 'black',
        icon: (
            <path d="M12,2C9.76,2 7.78,3.05 6.5,4.68L7.93,6.11C8.84,4.84 10.32,4 12,4A5,5 0 0,1 17,9C17,10.68 16.16,12.16 14.89,13.06L16.31,14.5C17.94,13.21 19,11.24 19,9A7,7 0 0,0 12,2M3.28,4L2,5.27L5.04,8.3C5,8.53 5,8.76 5,9C5,11.38 6.19,13.47 8,14.74V17A1,1 0 0,0 9,18H14.73L18.73,22L20,20.72L3.28,4M7.23,10.5L12.73,16H10V13.58C8.68,13 7.66,11.88 7.23,10.5M9,20V21A1,1 0 0,0 10,22H14A1,1 0 0,0 15,21V20H9Z" />
        ),
    },
    {
        type: 'manual',
        icon: (
            <path d="M12,4A4,4 0 0,1 16,8A4,4 0 0,1 12,12A4,4 0 0,1 8,8A4,4 0 0,1 12,4M12,14C16.42,14 20,15.79 20,18V20H4V18C4,15.79 7.58,14 12,14Z" />
        ),
    },
    {
        type: 'auto',
        icon: (
            <path d="M21,3V15.5A3.5,3.5 0 0,1 17.5,19A3.5,3.5 0 0,1 14,15.5A3.5,3.5 0 0,1 17.5,12C18.04,12 18.55,12.12 19,12.34V6.47L9,8.6V17.5A3.5,3.5 0 0,1 5.5,21A3.5,3.5 0 0,1 2,17.5A3.5,3.5 0 0,1 5.5,14C6.04,14 6.55,14.12 7,14.34V6L21,3Z" />
        ),
    },
];

function capitalizeFirstLetter(string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
}

function QrModal({ text, onClose }) {
    return (
        <div
            style="position:fixed;top:0px;left:0px;width:100vw;height:100vh;background-color:rgba(0,0,0,0.8);display:flex;align-items:center;justify-content:center;"
            onClick={onClose}
        >
            <div
                style="width: 50vw; background-color: #fff;"
                dangerouslySetInnerHTML={{ __html: encodeQR(text, 'svg') }}
            />
        </div>
    );
}

export function App() {
    const [selectedColor, setSelectedColor] = useState(undefined);
    const [selectedToggleColor, setSelectedToggleColor] = useState(undefined);
    const [selectedToggleSpeed, setSelectedToggleSpeed] = useState(undefined);
    const [selectedStrobeSpeed, setSelectedStrobeSpeed] = useState(undefined);
    const [selectedMode, setSelectedMode] = useState(undefined);
    const [showQrCode, setShowQrCode] = useState(false);

    useEffect(async () => {
        const { state } = await ipc.request('getState');
        setSelectedColor(state.color);
        setSelectedToggleColor(state.toggleColor);
        setSelectedToggleSpeed(state.toggleSpeed);
        setSelectedStrobeSpeed(state.strobeSpeed);
        setSelectedMode(state.mode);

        const selectedColorListener = ipc.on('setColor', ({ color }) => setSelectedColor(color));
        const selectedToggleColorListener = ipc.on('setToggleColor', ({ color }) => setSelectedToggleColor(color));
        const selectedToggleSpeedListener = ipc.on('setToggleSpeed', ({ speed }) => setSelectedToggleSpeed(speed));
        const selectedStrobeSpeedListener = ipc.on('setStrobeSpeed', ({ speed }) => setSelectedStrobeSpeed(speed));
        const selectedModeListener = ipc.on('setMode', ({ mode }) => setSelectedMode(mode));
        return () => {
            selectedColorListener.remove();
            selectedToggleColorListener.remove();
            selectedToggleSpeedListener.remove();
            selectedStrobeSpeedListener.remove();
            selectedModeListener.remove();
        };
    }, []);
    useEffect(async () => {
        if (selectedColor !== undefined) await ipc.send('setColor', { color: selectedColor });
    }, [selectedColor]);
    useEffect(async () => {
        if (selectedToggleColor !== undefined) await ipc.send('setToggleColor', { color: selectedToggleColor });
    }, [selectedToggleColor]);
    useEffect(async () => {
        if (selectedToggleSpeed !== undefined) await ipc.send('setToggleSpeed', { speed: selectedToggleSpeed });
    }, [selectedToggleSpeed]);
    useEffect(async () => {
        if (selectedStrobeSpeed !== undefined) await ipc.send('setStrobeSpeed', { speed: selectedStrobeSpeed });
    }, [selectedStrobeSpeed]);
    useEffect(async () => {
        if (selectedMode !== undefined) await ipc.send('setMode', { mode: selectedMode });
    }, [selectedMode]);

    return (
        <>
            <h2>Color</h2>
            <div className="button-grid">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        className={`color-button${color === selectedColor ? ' selected' : ''}`}
                        style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                        onClick={() => setSelectedColor(color)}
                    />
                ))}
            </div>

            <h2>Toggle Colors</h2>
            <div className="button-grid">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        className={`color-button${color === selectedToggleColor ? ' selected' : ''}`}
                        style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                        onClick={() => setSelectedToggleColor(color)}
                    />
                ))}
            </div>

            <h2>Toggle Speeds</h2>
            <div className="button-grid">
                {SPEEDS.map((speed) => (
                    <button
                        key={speed}
                        className={`text-button${speed === selectedToggleSpeed ? ' selected' : ''}`}
                        onClick={() => setSelectedToggleSpeed(speed)}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            <h2>Strobe Speeds</h2>
            <div className="button-grid">
                {SPEEDS.map((speed) => (
                    <button
                        key={speed}
                        className={`text-button${speed === selectedStrobeSpeed ? ' selected' : ''}`}
                        onClick={() => setSelectedStrobeSpeed(speed)}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            <div className="bottom-controls-container">
                {MODES.map((mode) => (
                    <button
                        key={mode}
                        className={`control-button ${mode.type === selectedMode ? ' selected' : ''}`}
                        onClick={() => setSelectedMode(mode.type)}
                    >
                        <svg className="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                            {mode.icon}
                        </svg>
                        {capitalizeFirstLetter(mode.type)}
                    </button>
                ))}
            </div>

            <div style="position:fixed;bottom:1rem;right:1rem;display:flex;">
                <a
                    class="button"
                    style="display:flex;align-items:center;justify-content:center;padding:.5rem;margin-right:.5rem"
                    href={window.location.href}
                    target="_blank"
                >
                    <svg className="icon" style="margin:0;" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M16.36,14C16.44,13.34 16.5,12.68 16.5,12C16.5,11.32 16.44,10.66 16.36,10H19.74C19.9,10.64 20,11.31 20,12C20,12.69 19.9,13.36 19.74,14M14.59,19.56C15.19,18.45 15.65,17.25 15.97,16H18.92C17.96,17.65 16.43,18.93 14.59,19.56M14.34,14H9.66C9.56,13.34 9.5,12.68 9.5,12C9.5,11.32 9.56,10.65 9.66,10H14.34C14.43,10.65 14.5,11.32 14.5,12C14.5,12.68 14.43,13.34 14.34,14M12,19.96C11.17,18.76 10.5,17.43 10.09,16H13.91C13.5,17.43 12.83,18.76 12,19.96M8,8H5.08C6.03,6.34 7.57,5.06 9.4,4.44C8.8,5.55 8.35,6.75 8,8M5.08,16H8C8.35,17.25 8.8,18.45 9.4,19.56C7.57,18.93 6.03,17.65 5.08,16M4.26,14C4.1,13.36 4,12.69 4,12C4,11.31 4.1,10.64 4.26,10H7.64C7.56,10.66 7.5,11.32 7.5,12C7.5,12.68 7.56,13.34 7.64,14M12,4.03C12.83,5.23 13.5,6.57 13.91,8H10.09C10.5,6.57 11.17,5.23 12,4.03M18.92,8H15.97C15.65,6.75 15.19,5.55 14.59,4.44C16.43,5.07 17.96,6.34 18.92,8M12,2C6.47,2 2,6.5 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
                    </svg>
                </a>

                <button
                    class="button"
                    style="display:flex;align-items:center;justify-content:center;padding:.5rem"
                    onClick={() => setShowQrCode(true)}
                >
                    <svg className="icon" style="margin:0;" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M3,11H5V13H3V11M11,5H13V9H11V5M9,11H13V15H11V13H9V11M15,11H17V13H19V11H21V13H19V15H21V19H19V21H17V19H13V21H11V17H15V15H17V13H15V11M19,19V15H17V19H19M15,3H21V9H15V3M17,5V7H19V5H17M3,3H9V9H3V3M5,5V7H7V5H5M3,15H9V21H3V15M5,17V19H7V17H5Z" />
                    </svg>
                </button>
            </div>

            {showQrCode && <QrModal text={window.location.href} onClose={() => setShowQrCode(false)} />}
        </>
    );
}
