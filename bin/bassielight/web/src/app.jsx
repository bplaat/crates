/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 750, 1000];

function send(type, data) {
    window.ipc.postMessage(JSON.stringify({ type, ...data }));
}

export function App() {
    const [selectedColor, setSelectedColor] = useState(COLORS[0]);
    const [selectedToggleColor, setSelectedToggleColor] = useState(COLORS[0]);
    const [selectedToggleSpeed, setSelectedToggleSpeed] = useState(SPEEDS[0]);
    const [selectedStrobeSpeed, setSelectedStrobeSpeed] = useState(SPEEDS[0]);

    return (
        <div className="container">
            <h1>
                BassieLight <span style={{ fontSize: '1rem', color: '#aaa' }}>v0.1.0</span>
            </h1>
            <p>
                <i>This is an early prototype, more features are coming soon...</i>
            </p>

            <h2>Color</h2>
            <div className="button-grid">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        className={`color-button${color === selectedColor ? ' selected' : ''}`}
                        style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                        onClick={() => {
                            setSelectedColor(color);
                            send('setColor', { color });
                        }}
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
                        onClick={() => {
                            setSelectedToggleColor(color);
                            send('setToggleColor', { color });
                        }}
                    />
                ))}
            </div>

            <h2>Toggle Speeds</h2>
            <div className="button-grid">
                {SPEEDS.map((speed) => (
                    <button
                        key={String(speed)}
                        className={`text-button${speed === selectedToggleSpeed ? ' selected' : ''}`}
                        onClick={() => {
                            setSelectedToggleSpeed(speed);
                            send('setToggleSpeed', { speed });
                        }}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            <h2>Strobe Speeds</h2>
            <div className="button-grid">
                {SPEEDS.map((speed) => (
                    <button
                        key={String(speed)}
                        className={`text-button${speed === selectedStrobeSpeed ? ' selected' : ''}`}
                        onClick={() => {
                            setSelectedStrobeSpeed(speed);
                            send('setStrobeSpeed', { speed });
                        }}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            <p className="footer">
                Made by <a href="https://bplaat.nl">Bastiaan van der Plaat</a>
            </p>
        </div>
    );
}
