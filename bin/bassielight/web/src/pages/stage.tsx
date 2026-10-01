/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useRef, useState } from 'preact/hooks';
import { IpcContext } from '../app.tsx';
import { GoboButtons } from '../components/gobo.tsx';
import {
    AccountIcon,
    ChartBellCurveCumulativeIcon,
    ChartLinearIcon,
    ChartStepIcon,
    CloseIcon,
    KindIcon,
    LightbulbOffIcon,
    MetronomeIcon,
    MusicIcon,
} from '../components/icons.tsx';
import { Visualization } from '../components/visualization.tsx';
import {
    $document,
    controlKinds,
    findProfile,
    selectedFixtureIds,
    selectionName,
    toggleScript,
    useDmxOutput,
    type ControlKind,
    type MovingHead,
    type Presets,
    type Selection,
} from '../stage.ts';
import { capitalize, colorToHex } from '../utils.ts';
import './stage.css';

const COLORS = [0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff];
// Speeds in beats of the shared tempo
// Speeds from slow to fast in beats of the shared tempo
type Speed = number | null;
const SPEEDS: Speed[] = [null, 8, 4, 2, 1, 1 / 2, 1 / 4, 1 / 8];
const FRACTIONS: Record<number, string> = { 2: '½', 4: '¼', 8: '⅛' };
const TAP_RESET_MS = 2000;
const TWEENS = [
    { type: 'direct', icon: ChartStepIcon },
    { type: 'linear', icon: ChartLinearIcon },
    { type: 'ease', icon: ChartBellCurveCumulativeIcon },
];
const MODES = [
    { type: 'black', icon: LightbulbOffIcon },
    { type: 'manual', icon: AccountIcon },
    { type: 'auto', icon: MusicIcon },
];

interface FixtureState {
    color: number;
    toggleColor: number;
    intensity: number;
    toggleTween: string;
    toggleSpeed: number | null;
    strobeSpeed: Speed;
    switchesToggle: boolean[];
    switchesPress: boolean[];
    flashOn: boolean;
    flashPress: boolean;
    flashIntensity: number;
    flashSpeed: number;
    preset: number | null;
    presetSpeed: number;
    gobo: number | null;
    focus: number;
    movement: number | null;
    movementSpeed: number;
}

type SwitchChange = { index: number; on: boolean };
type FixtureProp =
    | Partial<Omit<FixtureState, 'switchesToggle' | 'switchesPress'>>
    | { switchToggle: SwitchChange }
    | { switchPress: SwitchChange };

const DEFAULT_FIXTURE_STATE: FixtureState = {
    color: 0x000000,
    toggleColor: 0x000000,
    intensity: 1,
    toggleTween: 'direct',
    toggleSpeed: null,
    strobeSpeed: null,
    switchesToggle: [false, false, false, false],
    switchesPress: [false, false, false, false],
    flashOn: false,
    flashPress: false,
    flashIntensity: 1,
    flashSpeed: 0.5,
    preset: null,
    presetSpeed: 0.5,
    gobo: null,
    focus: 0,
    movement: null,
    movementSpeed: 0.5,
};

function applyFixtureProp(state: FixtureState, prop: FixtureProp): FixtureState {
    const setSwitch = (switches: boolean[], { index, on }: SwitchChange) =>
        switches.map((value, i) => (i === index ? on : value));
    if ('switchToggle' in prop) return { ...state, switchesToggle: setSwitch(state.switchesToggle, prop.switchToggle) };
    if ('switchPress' in prop) return { ...state, switchesPress: setSwitch(state.switchesPress, prop.switchPress) };
    return { ...state, ...prop };
}

const KIND_LABELS: Record<ControlKind, string> = {
    rgb: 'RGB',
    preset: 'Presets',
    movingHead: 'Moving heads',
    switch: 'Switches',
    strobe: 'Strobes',
};

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

/// Every tap moves the downbeat to now, two or more taps in a row also set the BPM
function TapTempoButton({ bpm, onTap }: { bpm: number | undefined; onTap: (bpm: number) => void }) {
    const taps = useRef<number[]>([]);
    return (
        <button
            class="button is-expanded"
            title="Tap tempo"
            onClick={() => {
                const now = window.performance.now();
                if (taps.current.length > 0 && now - taps.current[taps.current.length - 1] > TAP_RESET_MS) {
                    taps.current = [];
                }
                taps.current.push(now);
                if (taps.current.length > 4) taps.current.shift();
                const intervals = taps.current.length - 1;
                const average = (taps.current[intervals] - taps.current[0]) / intervals;
                if (intervals > 0) onTap(Math.round(60000 / average));
                else if (bpm) onTap(bpm);
            }}
        >
            <MetronomeIcon />
            {bpm ? `${bpm} BPM` : 'BPM'}
        </button>
    );
}

function beatsLabel(beats: number): string {
    return beats < 1 ? FRACTIONS[Math.round(1 / beats)] : `${beats}`;
}

function RgbControls({ state, setProp }: { state: FixtureState; setProp: (prop: FixtureProp) => void }) {
    return (
        <>
            <h2 class="title">Color</h2>
            <div class="buttons">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        class={`swatch ${color === state.color ? 'is-selected' : ''}`}
                        style={{ backgroundColor: colorToHex(color) }}
                        onClick={() => setProp({ color })}
                    />
                ))}
            </div>

            <h2 class="title">Toggle Color</h2>
            <div class="buttons">
                {COLORS.map((color) => (
                    <button
                        key={color}
                        class={`swatch ${color === state.toggleColor ? 'is-selected' : ''}`}
                        style={{ backgroundColor: colorToHex(color) }}
                        onClick={() => setProp({ toggleColor: color })}
                    />
                ))}
            </div>

            <h2 class="title">Intensity</h2>
            <Slider value={state.intensity} onChange={(intensity) => setProp({ intensity })} />

            <h2 class="title">Toggle Tween</h2>
            <div class="buttons">
                {TWEENS.map((tween) => (
                    <button
                        key={tween.type}
                        class={`button ${tween.type === state.toggleTween ? 'is-selected' : ''}`}
                        onClick={() => setProp({ toggleTween: tween.type })}
                        title={capitalize(tween.type)}
                    >
                        <tween.icon />
                    </button>
                ))}
            </div>

            <h2 class="title">Toggle Speed</h2>
            <SpeedButtons speed={state.toggleSpeed} onChange={(toggleSpeed) => setProp({ toggleSpeed })} />

            <h2 class="title">Strobe Speed</h2>
            <SpeedButtons speed={state.strobeSpeed} onChange={(strobeSpeed) => setProp({ strobeSpeed })} />
        </>
    );
}

function SwitchControls({
    labels,
    state,
    setProp,
}: {
    labels: string[];
    state: FixtureState;
    setProp: (prop: FixtureProp) => void;
}) {
    return (
        <>
            <h2 class="title">Toggle</h2>
            <div class="buttons is-grid">
                {labels.map((label, index) => (
                    <button
                        key={index}
                        class={`button is-pill ${state.switchesToggle[index] ? 'is-selected' : ''}`}
                        title={label || `Toggle ${index + 1}`}
                        onClick={() => setProp({ switchToggle: { index, on: !state.switchesToggle[index] } })}
                    >
                        <span class="button-label">{label || index + 1}</span>
                    </button>
                ))}
            </div>

            <h2 class="title">Press</h2>
            <div class="buttons is-grid">
                {labels.map((label, index) => (
                    <button
                        key={index}
                        class={`button is-pill ${state.switchesPress[index] ? 'is-selected' : ''}`}
                        title={label || `Press ${index + 1}`}
                        onPointerDown={() => setProp({ switchPress: { index, on: true } })}
                        onPointerUp={(event: PointerEvent) => {
                            setProp({ switchPress: { index, on: false } });
                            (event.currentTarget as HTMLElement).blur();
                        }}
                    >
                        <span class="button-label">{label || index + 1}</span>
                    </button>
                ))}
            </div>
        </>
    );
}

function StrobeControls({ state, setProp }: { state: FixtureState; setProp: (prop: FixtureProp) => void }) {
    return (
        <>
            <div class="buttons is-grid is-two">
                <button
                    class={`button is-pill ${state.flashOn ? 'is-selected' : ''}`}
                    onClick={() => setProp({ flashOn: !state.flashOn })}
                >
                    {state.flashOn ? 'On' : 'Off'}
                </button>
                <button
                    class={`button is-pill ${state.flashPress ? 'is-selected' : ''}`}
                    onPointerDown={() => setProp({ flashPress: true })}
                    onPointerUp={(event: PointerEvent) => {
                        setProp({ flashPress: false });
                        (event.currentTarget as HTMLElement).blur();
                    }}
                    onPointerLeave={() => state.flashPress && setProp({ flashPress: false })}
                >
                    Flash
                </button>
            </div>

            <h2 class="title">Intensity</h2>
            <Slider value={state.flashIntensity} onChange={(flashIntensity) => setProp({ flashIntensity })} />

            <h2 class="title">Flash Speed</h2>
            <Slider value={state.flashSpeed} onChange={(flashSpeed) => setProp({ flashSpeed })} />
        </>
    );
}

function PresetControls({
    presets,
    state,
    setProp,
}: {
    presets: Presets;
    state: FixtureState;
    setProp: (prop: FixtureProp) => void;
}) {
    return (
        <>
            <h2 class="title">Preset</h2>
            <select
                class="input"
                value={state.preset ?? ''}
                onChange={(e) =>
                    setProp({ preset: e.currentTarget.value === '' ? null : Number(e.currentTarget.value) })
                }
            >
                <option value="">Own color</option>
                {presets.list.map((preset, index) => (
                    <option key={index} value={index}>
                        {preset.name}
                    </option>
                ))}
            </select>

            <h2 class="title">Preset Speed</h2>
            <Slider value={state.presetSpeed} onChange={(presetSpeed) => setProp({ presetSpeed })} />
        </>
    );
}

function MovingHeadControls({
    movingHead,
    state,
    setProp,
}: {
    movingHead: MovingHead;
    state: FixtureState;
    setProp: (prop: FixtureProp) => void;
}) {
    return (
        <>
            <h2 class="title">Gobo</h2>
            <GoboButtons gobos={movingHead.gobos} selected={state.gobo} onSelect={(gobo) => setProp({ gobo })} />

            <h2 class="title">Focus (Big to Small)</h2>
            <Slider value={state.focus} onChange={(focus) => setProp({ focus })} />

            <h2 class="title">Movement</h2>
            <select
                class="input"
                value={state.movement ?? ''}
                onChange={(e) =>
                    setProp({ movement: e.currentTarget.value === '' ? null : Number(e.currentTarget.value) })
                }
            >
                <option value="">Stand still</option>
                {movingHead.movements.map((movement, index) => (
                    <option key={index} value={index}>
                        {movement.name}
                    </option>
                ))}
            </select>

            <h2 class="title">Movement Speed</h2>
            <Slider value={state.movementSpeed} onChange={(movementSpeed) => setProp({ movementSpeed })} />
        </>
    );
}

function Slider({ value, onChange }: { value: number; onChange: (value: number) => void }) {
    return (
        <input
            class="slider"
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={value}
            onInput={(e) => onChange(parseFloat(e.currentTarget.value))}
        />
    );
}

function SpeedButtons({ speed, onChange }: { speed: Speed; onChange: (speed: Speed) => void }) {
    const label = (value: Speed) => (value === null ? 'Off' : beatsLabel(value));
    const title = (value: Speed) => (value === null ? 'Off' : `${beatsLabel(value)} beat`);
    return (
        <div class="buttons is-grid">
            {SPEEDS.map((value) => (
                <button
                    key={value}
                    class={`button is-pill ${value === speed ? 'is-selected' : ''}`}
                    title={title(value)}
                    onClick={() => onChange(value)}
                >
                    {label(value)}
                </button>
            ))}
        </div>
    );
}

export function StagePage() {
    const ipc = useContext(IpcContext)!;

    const [selection, setSelection] = useState<Selection>(null);
    const [fixtureStates, setFixtureStates] = useState<Record<number, FixtureState>>({});
    const [selectedMode, setSelectedMode] = useIpcState('mode');
    const [bpm, setBpm] = useIpcState('bpm');

    const applyProp = (fixtures: number[], prop: FixtureProp) =>
        setFixtureStates((states) => {
            const newStates = { ...states };
            for (const id of fixtures) newStates[id] = applyFixtureProp(states[id] ?? DEFAULT_FIXTURE_STATE, prop);
            return newStates;
        });

    useEffect(() => {
        document.title = 'BassieLight - Stage';

        const listener = ipc.on('setFixtureProp', ({ fixtures, prop }: any) => applyProp(fixtures, prop));
        return () => listener.remove();
    }, []);
    // Freeze the setup while performing, changes to the stage folder are picked up afterwards
    const fixtureOutputs = useDmxOutput(ipc, true);

    // (Re)load the DMX state when a stage file is opened, the app resets fixture state then
    const path = $document.value?.path;
    useEffect(() => {
        setSelection(null);
        (async () => {
            const { state } = (await ipc.request('getState')) as {
                state: {
                    mode: string;
                    bpm: number;
                    fixtures: Record<number, FixtureState>;
                };
            };
            setFixtureStates(state.fixtures);
            setSelectedMode(state.mode, false);
            setBpm(Math.round(state.bpm), false);
        })();
    }, [path]);

    if (!$document.value) return null;
    const { stage, fixtureTypes } = $document.value;

    // Without a selection all fixtures are controlled, a removed fixture or group falls back to that
    const name = selectionName(stage, selection);
    const activeSelection = name !== undefined ? selection : null;
    const targetIds = selectedFixtureIds(stage, activeSelection);
    const targets = stage.fixtures.filter((f) => targetIds.includes(f.id));

    // Controls per kind in the selection, each only changes the fixtures of its kind
    const sections = (Object.keys(KIND_LABELS) as ControlKind[])
        .map((kind) => ({
            kind,
            fixtures: targets.filter((f) => controlKinds(findProfile(fixtureTypes, f.type)).includes(kind)),
        }))
        .filter(({ fixtures }) => fixtures.length > 0)
        .map(({ kind, fixtures }) => {
            const ids = fixtures.map((f) => f.id);
            const state = fixtureStates[ids[0]] ?? DEFAULT_FIXTURE_STATE;
            const setProp = (prop: FixtureProp) => {
                applyProp(ids, prop);
                ipc.send('setFixtureProp', { fixtures: ids, prop });
            };
            const { presets, movingHead } = findProfile(fixtureTypes, fixtures[0].type);
            return { kind, count: fixtures.length, labels: fixtures[0].switches, presets, movingHead, state, setProp };
        });

    return (
        <>
            <div class="main">
                <Visualization
                    stage={stage}
                    fixtureTypes={fixtureTypes}
                    outputs={fixtureOutputs}
                    mode={selectedMode}
                    onScriptToggle={(name) => toggleScript(ipc, name)}
                    selection={activeSelection}
                    onSelect={setSelection}
                />

                <div class="buttons is-centered">
                    {MODES.map((mode) => (
                        <button
                            key={mode.type}
                            class={`button is-expanded ${mode.type === selectedMode ? 'is-selected' : ''}`}
                            onClick={() => setSelectedMode(mode.type)}
                        >
                            <mode.icon />
                            {capitalize(mode.type)}
                        </button>
                    ))}
                    <TapTempoButton bpm={bpm} onTap={setBpm} />
                </div>
            </div>

            <div class="sidebar">
                <div class="selection">
                    <div class="selection-name">
                        <span class="field-label">{activeSelection ? capitalize(activeSelection.type) : 'Room'}</span>
                        <strong>{name ?? 'All fixtures'}</strong>
                    </div>
                    {activeSelection && (
                        <button class="icon-button" title="Select room" onClick={() => setSelection(null)}>
                            <CloseIcon />
                        </button>
                    )}
                </div>

                {sections.length === 0 && <p class="block">No fixtures to control</p>}
                {sections.map(({ kind, count, labels, presets, movingHead, state, setProp }) => (
                    <section key={kind}>
                        {sections.length > 1 && (
                            <div class="kind-bar">
                                <KindIcon kind={kind} />
                                {KIND_LABELS[kind]}
                                <span class="kind-bar-count">{count}</span>
                            </div>
                        )}
                        {kind === 'rgb' && <RgbControls state={state} setProp={setProp} />}
                        {kind === 'movingHead' && movingHead && (
                            <MovingHeadControls movingHead={movingHead} state={state} setProp={setProp} />
                        )}
                        {kind === 'preset' && presets && (
                            <PresetControls presets={presets} state={state} setProp={setProp} />
                        )}
                        {kind === 'switch' && (
                            <SwitchControls labels={labels ?? ['', '', '', '']} state={state} setProp={setProp} />
                        )}
                        {kind === 'strobe' && <StrobeControls state={state} setProp={setProp} />}
                    </section>
                ))}
            </div>
        </>
    );
}
