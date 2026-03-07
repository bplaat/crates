/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect, useContext, useRef } from 'preact/hooks';
import { AccountIcon, LightbulbOffIcon, MusicIcon, TweenDirect, TweenEase, TweenLinear } from '../components/icons.tsx';
import { capitalize } from '../utils.ts';
import { IpcContext } from '../app.tsx';
import { $dmxLive } from '../components/menubar.tsx';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
const SPEEDS = [null, 22, 50, 100, 200, 250, 500, 1000];
const TWEENS = [
    { type: 'direct', icon: TweenDirect },
    { type: 'linear', icon: TweenLinear },
    { type: 'ease', icon: TweenEase },
];
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

function TapTempoButton({
    selectedSpeed,
    onSpeedChange,
}: {
    selectedSpeed: number | null;
    onSpeedChange: (ms: number) => void;
}) {
    const taps = useRef<number[]>([]);
    const isSelected = !SPEEDS.includes(selectedSpeed);
    const bpm = isSelected && selectedSpeed ? Math.round(60000 / selectedSpeed) : null;

    return (
        <button
            class={`flex items-center gap-2 justify-center min-w-20 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${isSelected ? 'border-blue-500' : 'border-transparent'}`}
            onClick={() => {
                const now = window.performance.now();
                if (taps.current.length > 0 && now - taps.current[taps.current.length - 1] > 2000) {
                    taps.current = [];
                }
                taps.current.push(now);
                if (taps.current.length > 4) {
                    taps.current.shift();
                }
                if (taps.current.length > 1) {
                    let sum = 0;
                    for (let i = 1; i < taps.current.length; i++) {
                        sum += taps.current[i] - taps.current[i - 1];
                    }
                    const avg = sum / (taps.current.length - 1);
                    onSpeedChange(Math.round(avg));
                }
            }}
        >
            {bpm ? `${bpm}BPM` : 'BPM'}
        </button>
    );
}

export function StagePage() {
    const ipc = useContext(IpcContext)!;

    const [selectedColor, setSelectedColor] = useIpcState('color');
    const [selectedToggleColor, setSelectedToggleColor] = useIpcState('toggleColor');
    const [intensity, setIntensity] = useIpcState('intensity');
    const [selectedToggleTween, setSelectedToggleTween] = useIpcState('toggleTween');
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
                    toggleTween: string;
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
            setSelectedToggleTween(state.toggleTween, false);
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
            <div class="flex-1 flex flex-col">
                <div class="flex-1" />

                <div class="flex flex-wrap gap-2 my-4 items-center justify-center">
                    {MODES.map((mode) => (
                        <button
                            key={mode}
                            class={`flex items-center gap-2 justify-center flex-1 max-w-48 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${mode.type === selectedMode ? 'border-blue-500' : 'border-transparent'}`}
                            onClick={() => setSelectedMode(mode.type)}
                        >
                            <mode.icon />
                            {capitalize(mode.type)}
                        </button>
                    ))}
                </div>
            </div>

            <div class="w-80 bg-zinc-800 p-4 overflow-y-auto">
                <h2 class="font-bold mt-0 mb-4 text-[1.35rem] border-b border-zinc-600 pb-2">Color</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    {COLORS.map((color) => (
                        <button
                            key={color}
                            class={`w-14 h-14 rounded-lg border-2 cursor-pointer transition-all active:scale-95 ${color === selectedColor ? 'border-white' : 'border-white/10 hover:border-white'}`}
                            style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                            onClick={() => setSelectedColor(color)}
                        />
                    ))}
                </div>

                <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Toggle Color</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    {COLORS.map((color) => (
                        <button
                            key={color}
                            class={`w-14 h-14 rounded-lg border-2 cursor-pointer transition-all active:scale-95 ${color === selectedToggleColor ? 'border-white' : 'border-white/10 hover:border-white'}`}
                            style={{ backgroundColor: `#${color.toString(16).padStart(6, '0')}` }}
                            onClick={() => setSelectedToggleColor(color)}
                        />
                    ))}
                </div>

                <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Intensity</h2>
                <input
                    class="slider"
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={intensity ?? 0}
                    onInput={(e) => setIntensity(parseFloat((e.target as HTMLInputElement).value))}
                />

                <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Toggle Tween</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    {TWEENS.map((tween) => (
                        <button
                            key={tween.type}
                            class={`p-2 border-2 rounded-lg bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${tween.type === selectedToggleTween ? 'border-blue-500' : 'border-transparent'}`}
                            onClick={() => setSelectedToggleTween(tween.type)}
                            title={capitalize(tween.type)}
                        >
                            <tween.icon />
                        </button>
                    ))}
                </div>

                <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Toggle Speed</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    {SPEEDS.map((speed) => (
                        <button
                            key={speed}
                            class={`flex items-center gap-2 justify-center min-w-20 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${speed === selectedToggleSpeed ? 'border-blue-500' : 'border-transparent'}`}
                            onClick={() => setSelectedToggleSpeed(speed)}
                        >
                            {speed == null ? 'Off' : `${speed}ms`}
                        </button>
                    ))}
                    <TapTempoButton selectedSpeed={selectedToggleSpeed} onSpeedChange={setSelectedToggleSpeed} />
                </div>

                <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Strobe Speed</h2>
                <div class="flex flex-wrap gap-2 mb-4">
                    {SPEEDS.map((speed) => (
                        <button
                            key={speed}
                            class={`flex items-center gap-2 justify-center min-w-20 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${speed === selectedStrobeSpeed ? 'border-blue-500' : 'border-transparent'}`}
                            onClick={() => setSelectedStrobeSpeed(speed)}
                        >
                            {speed == null ? 'Off' : `${speed}ms`}
                        </button>
                    ))}
                    <TapTempoButton selectedSpeed={selectedStrobeSpeed} onSpeedChange={setSelectedStrobeSpeed} />
                </div>

                {switchesLabels && (
                    <>
                        <h2 class="font-bold my-4 text-[1.35rem] border-b border-zinc-600 pb-2">Switches</h2>
                        <p class="my-4">Toggle</p>
                        <div class="flex flex-wrap gap-2 mb-4">
                            {switchesLabels.map((label, index) => (
                                <button
                                    key={`toggle-${index}`}
                                    class={`flex items-center gap-2 justify-center min-w-20 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${switchesToggle[index] ? 'border-blue-500' : 'border-transparent'}`}
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

                        <p class="my-4">Press</p>
                        <div class="flex flex-wrap gap-2 mb-4">
                            {switchesLabels.map((label, index) => (
                                <button
                                    key={`press-${index}`}
                                    class={`flex items-center gap-2 justify-center min-w-20 h-12 px-2 border-2 rounded-lg font-medium bg-zinc-700 text-zinc-200 cursor-pointer transition-all hover:bg-zinc-600 hover:border-blue-500 active:bg-zinc-500 active:scale-95 outline-none ${switchesPress[index] ? 'border-blue-500' : 'border-transparent'}`}
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
