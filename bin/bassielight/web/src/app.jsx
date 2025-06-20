/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect } from 'preact/hooks';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 750, 1000];
const MODES = ["black", "manual", "auto"];

function send(type, data) {
    window.ipc.postMessage(JSON.stringify({ type, ...data }));
}

function capitalizeFirstLetter(string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
}

export function App() {
    const [selectedColor, setSelectedColor] = useState(COLORS[0]);
    const [selectedToggleColor, setSelectedToggleColor] = useState(COLORS[0]);
    const [selectedToggleSpeed, setSelectedToggleSpeed] = useState(SPEEDS[0]);
    const [selectedStrobeSpeed, setSelectedStrobeSpeed] = useState(SPEEDS[0]);
    const [selectedMode, setSelectedMode] = useState(MODES[0]);

    useEffect(() => {
        send('setColor', { color: selectedColor });
    }, [selectedColor]);

    useEffect(() => {
        send('setToggleColor', { color: selectedToggleColor });
    }, [selectedToggleColor]);

    useEffect(() => {
        send('setToggleSpeed', { speed: selectedToggleSpeed });
    }, [selectedToggleSpeed]);

    useEffect(() => {
        send('setStrobeSpeed', { speed: selectedStrobeSpeed });
    }, [selectedStrobeSpeed]);

    useEffect(() => {
        send('setMode', { mode: selectedMode });
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
                        className={`control-button ${mode === selectedMode ? ' selected' : ''}`}
                        onClick={() => setSelectedMode(mode)}
                    >
                        {capitalizeFirstLetter(mode)}
                    </button>
                ))}
            </div>
        </>
    );
}
