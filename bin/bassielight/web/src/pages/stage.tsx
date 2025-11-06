/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect, useContext } from 'preact/hooks';
import { AccountIcon, LightbulbOffIcon, MusicIcon } from '../components/icons.tsx';
import { capitalize } from '../utils.ts';
import { IpcContext } from '../app.tsx';
import { $dmxLive } from '../components/menubar.tsx';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 750, 1000];
const MODES = [
    { type: 'black', icon: LightbulbOffIcon },
    { type: 'manual', icon: AccountIcon },
    { type: 'auto', icon: MusicIcon },
];

function useIpcState(key: string): [any, (value: any, isUserInitiated?: boolean) => void] {
    const ipc = useContext(IpcContext)!;
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

export function StagePage() {
    const ipc = useContext(IpcContext)!;

    const [selectedColor, setSelectedColor] = useIpcState('color');
    const [selectedToggleColor, setSelectedToggleColor] = useIpcState('toggleColor');
    const [intensity, setIntensity] = useIpcState('intensity');
    const [selectedToggleSpeed, setSelectedToggleSpeed] = useIpcState('toggleSpeed');
    const [selectedStrobeSpeed, setSelectedStrobeSpeed] = useIpcState('strobeSpeed');
    const [switchesLabels, setSwitchesLabels] = useState<string[] | null>(null);
    const [switchesToggle, setSwitchesToggle] = useIpcState('switchesToggle');
    const [switchesPress, setSwitchesPress] = useIpcState('switchesPress');
    const [selectedMode, setSelectedMode] = useIpcState('mode');

    useEffect(() => {
        document.title = 'BassieLight - Stage';

        // Load initial state
        (async () => {
            const { state } = (await ipc.request('getState')) as {
                state: {
                    color: number;
                    toggleColor: number;
                    intensity: number;
                    toggleSpeed: number | null;
                    strobeSpeed: number | null;
                    mode: string;
                    switchesLabels: string[] | null;
                    switchesToggle: boolean[];
                    switchesPress: boolean[];
                };
            };
            setSelectedColor(state.color, false);
            setSelectedToggleColor(state.toggleColor, false);
            setIntensity(state.intensity, false);
            setSelectedToggleSpeed(state.toggleSpeed, false);
            setSelectedStrobeSpeed(state.strobeSpeed, false);
            setSelectedMode(state.mode, false);
            setSwitchesLabels(state.switchesLabels);
            setSwitchesToggle(state.switchesToggle, false);
            setSwitchesPress(state.switchesPress, false);
        })();

        // Start DMX on mount, stop on unmount
        ipc.send('start');
        $dmxLive.value = true;
        return () => {
            ipc.send('stop');
            $dmxLive.value = false;
        };
    }, []);

    return (
        <>
            <div class="main">
                <div class="flex"></div>

                <div class="buttons is-centered">
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
            </div>

            <div class="sidebar">
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

                <h2 class="subtitle">Intensity</h2>
                <input
                    class="slider is-fullwidth"
                    type="range"
                    min="0"
                    max="255"
                    value={intensity ?? 0}
                    onInput={(e) => setIntensity(parseInt((e.target as HTMLInputElement).value, 10))}
                />

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
                        <p>Toggle</p>
                        <div class="buttons">
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

                        <p>Press</p>
                        <div class="buttons">
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
            </div>
        </>
    );
}
