/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect } from 'preact/hooks';
import { QrModal } from './components/qrmodal.tsx';
import { AccountIcon, LightbulbOffIcon, MusicIcon } from './components/icons.tsx';
import { capitalize } from './utils.ts';
import { ipc } from './index.tsx';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 750, 1000];
const MODES = [
    { type: 'black', icon: LightbulbOffIcon },
    { type: 'manual', icon: AccountIcon },
    { type: 'auto', icon: MusicIcon },
];

function useIpcState(key: string): [any, (value: any, isUserInitiated?: boolean) => void] {
    const [value, setValue] = useState(undefined);
    const setMessageType = `set${capitalize(key)}`;
    useEffect(() => {
        const listener = ipc.on(setMessageType, (data: { [key: string]: any }) => {
            setValue(data[key]);
        });
        return () => listener.remove();
    }, []);
    const setIpcValue = (newValue: any, isUserInitiated = true) => {
        setValue(newValue);
        if (isUserInitiated) {
            ipc.send(setMessageType, { [key]: newValue });
        }
    };
    return [value, setIpcValue];
}

const classes = css`
    .qr-button {
        position: fixed;
        bottom: 1rem;
        right: 1rem;
    }
`;

export function App() {
    const [showQrCode, setShowQrCode] = useState(false);
    const [switchesLabels, setSwitchesLabels] = useState<string[] | undefined>(undefined);
    const [selectedColor, setSelectedColor] = useIpcState('color');
    const [selectedToggleColor, setSelectedToggleColor] = useIpcState('toggleColor');
    const [selectedToggleSpeed, setSelectedToggleSpeed] = useIpcState('toggleSpeed');
    const [selectedStrobeSpeed, setSelectedStrobeSpeed] = useIpcState('strobeSpeed');
    const [switchesToggle, setSwitchesToggle] = useIpcState('switchesToggle');
    const [switchesPress, setSwitchesPress] = useIpcState('switchesPress');
    const [selectedMode, setSelectedMode] = useIpcState('mode');

    useEffect(() => {
        (async () => {
            const { state } = (await ipc.request('getState')) as {
                state: {
                    config: {
                        fixtures: {
                            type: string;
                            switches?: string[];
                        }[];
                    };
                    color: number;
                    toggleColor: number;
                    toggleSpeed: number | null;
                    strobeSpeed: number | null;
                    mode: string;
                    switchesToggle: boolean[];
                    switchesPress: boolean[];
                };
            };
            const switchFixture = state.config.fixtures.find((fixture) => fixture.type === 'multidim_mkii');
            if (switchFixture?.switches) {
                setSwitchesLabels(switchFixture.switches);
            }
            setSelectedColor(state.color, false);
            setSelectedToggleColor(state.toggleColor, false);
            setSelectedToggleSpeed(state.toggleSpeed, false);
            setSelectedStrobeSpeed(state.strobeSpeed, false);
            setSelectedMode(state.mode, false);
            setSwitchesToggle(state.switchesToggle, false);
            setSwitchesPress(state.switchesPress, false);
        })();
    }, []);

    return (
        <>
            <h2 class="subtitle">Color</h2>
            <div class="buttons">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        class={`button is-color ${color === selectedColor ? 'is-selected' : ''}`}
                        style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                        onClick={() => setSelectedColor(color)}
                    />
                ))}
            </div>

            <h2 class="subtitle">Toggle Colors</h2>
            <div class="buttons">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        class={`button is-color ${color === selectedToggleColor ? 'is-selected' : ''}`}
                        style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                        onClick={() => setSelectedToggleColor(color)}
                    />
                ))}
            </div>

            <h2 class="subtitle">Toggle Speeds</h2>
            <div class="buttons">
                {SPEEDS.map((speed) => (
                    <button
                        key={speed}
                        class={`button is-text ${speed === selectedToggleSpeed ? 'is-selected' : ''}`}
                        onClick={() => setSelectedToggleSpeed(speed)}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            <h2 class="subtitle">Strobe Speeds</h2>
            <div class="buttons">
                {SPEEDS.map((speed) => (
                    <button
                        key={speed}
                        class={`button is-text ${speed === selectedStrobeSpeed ? 'is-selected' : ''}`}
                        onClick={() => setSelectedStrobeSpeed(speed)}
                    >
                        {speed == null ? 'Off' : `${speed}ms`}
                    </button>
                ))}
            </div>

            {switchesLabels && (
                <>
                    <h2 class="subtitle">Switches</h2>
                    <div class="buttons">
                        Toggle
                        {switchesLabels.map((label, index) => (
                            <button
                                key={`toggle-${index}`}
                                class={`button is-text ${switchesToggle[index] ? 'is-selected' : ''}`}
                                onClick={() => {
                                    const newToggles = [...switchesToggle];
                                    newToggles[index] = !newToggles[index];
                                    setSwitchesToggle(newToggles);
                                }}
                            >
                                {label || `Toggle ${index + 1}`}
                            </button>
                        ))}
                    </div>
                    <div class="buttons">
                        Press
                        {switchesLabels.map((label, index) => (
                            <button
                                key={`press-${index}`}
                                class={`button is-text ${switchesPress[index] ? 'is-selected' : ''}`}
                                onMouseDown={() => {
                                    const newPresses = [...switchesPress];
                                    newPresses[index] = true;
                                    setSwitchesPress(newPresses);
                                }}
                                onMouseUp={(event: MouseEvent) => {
                                    const newPresses = [...switchesPress];
                                    newPresses[index] = false;
                                    setSwitchesPress(newPresses);
                                    (event.currentTarget as HTMLElement).blur();
                                }}
                            >
                                {label || `Press ${index + 1}`}
                            </button>
                        ))}
                    </div>
                </>
            )}

            <div class="buttons is-bottom">
                {MODES.map((mode) => (
                    <button
                        key={mode}
                        class={`button is-text is-large ${mode.type === selectedMode ? 'is-selected' : ''}`}
                        onClick={() => setSelectedMode(mode.type)}
                    >
                        <mode.icon />
                        {capitalize(mode.type)}
                    </button>
                ))}
            </div>

            <button class={`button is-icon ${classes['qr-button']}`} onClick={() => setShowQrCode(true)}>
                <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                    <path d="M3,11H5V13H3V11M11,5H13V9H11V5M9,11H13V15H11V13H9V11M15,11H17V13H19V11H21V13H19V15H21V19H19V21H17V19H13V21H11V17H15V15H17V13H15V11M19,19V15H17V19H19M15,3H21V9H15V3M17,5V7H19V5H17M3,3H9V9H3V3M5,5V7H7V5H5M3,15H9V21H3V15M5,17V19H7V17H5Z" />
                </svg>
            </button>

            {showQrCode && <QrModal contents={window.location.href} onClose={() => setShowQrCode(false)} />}
        </>
    );
}
