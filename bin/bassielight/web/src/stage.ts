/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { useEffect, useState } from 'preact/hooks';
import type { Ipc } from './ipc.ts';

export type Channel =
    | {
          type:
              | 'red'
              | 'green'
              | 'blue'
              | 'dimmer'
              | 'switch'
              | 'speed'
              | 'preset'
              | 'pan'
              | 'tilt'
              | 'colorWheel'
              | 'gobo'
              | 'focus'
              | 'movement'
              | 'movementSpeed'
              | 'shutter'
              | 'lampOn'
              | 'unused';
      }
    | { type: 'music'; value: number };

export type FixtureKind = 'rgb' | 'switch' | 'strobe' | 'movingHead';

/// Groups of controls in the stage sidebar, a fixture can have more than one
export type ControlKind = FixtureKind | 'preset';

export interface Presets {
    channels: Channel[];
    list: { name: string; color: number | null }[];
}

export interface FixtureProfile {
    type: string;
    name: string;
    kind: FixtureKind;
    /// Front size in millimeters, width and height
    size: [number, number];
    channels: Channel[];
    presets?: Presets;
    movingHead?: MovingHead;
}

export interface Gobo {
    name: string;
    shape: string;
}

export interface MovingHead {
    gobos: Gobo[];
    movements: { name: string }[];
}

export type FixtureOutput =
    | { rgb: number | null }
    | { movingHead: { color: number | null; gobo: string | null } }
    | { switch: boolean[] }
    | { strobe: { intensity: number; speed: number } };

export interface Fixture {
    id: number;
    name: string;
    type: string;
    addr: number;
    x: number;
    y: number;
    switches?: string[];
}

export interface Group {
    id: number;
    name: string;
    fixtures: number[];
    hide_outline?: boolean;
}

/// Rect in the room that selects its target when pressed, positioned by its center
export interface Button {
    id: number;
    label: string;
    x: number;
    y: number;
    width: number;
    height: number;
    target: ButtonTarget | null;
}

export interface Stage {
    room: { width: number; height: number };
    fixtures: Fixture[];
    groups: Group[];
    buttons: Button[];
}

export interface StageDocument {
    dirty: boolean;
    path: string;
    stage: Stage;
    fixtureTypes: FixtureProfile[];
    dmxLength: number;
}

export type Target = { type: 'fixture' | 'group'; id: number };
/// Buttons select a fixture or group, or start and stop a script
export type ButtonTarget = Target | { type: 'script'; name: string };
export type Selection = Target | { type: 'button'; id: number } | null;

/// Offset for pasted fixtures, in centimeters
export function pasteOffset(room: { width: number; height: number }): number {
    return Math.round(Math.max(room.width, room.height) / 20);
}

export function nextId(items: { id: number }[]): number {
    return Math.max(0, ...items.map((item) => item.id)) + 1;
}

export function selectedFixtureIds(stage: Stage, selection: Selection): number[] {
    if (selection?.type === 'fixture') return [selection.id];
    if (selection?.type === 'group') return stage.groups.find((g) => g.id === selection.id)?.fixtures ?? [];
    if (selection?.type === 'button') return [];
    return stage.fixtures.map((f) => f.id);
}

/// Fixtures highlighted in the visualization while the others are dimmed, `null` highlights all
export function highlightedFixtureIds(stage: Stage, selection: Selection): number[] | null {
    const target =
        selection?.type === 'button' ? (stage.buttons.find((b) => b.id === selection.id)?.target ?? null) : selection;
    return target && target.type !== 'script' ? selectedFixtureIds(stage, target) : null;
}

export function selectionName(stage: Stage, selection: Selection): string | undefined {
    if (selection?.type === 'button') {
        const button = stage.buttons.find((b) => b.id === selection.id);
        return button && buttonLabel(stage, button);
    }
    const items: { id: number; name: string }[] = selection?.type === 'group' ? stage.groups : stage.fixtures;
    return items.find((item) => item.id === selection?.id)?.name;
}

export function buttonLabel(stage: Stage, button: Button): string {
    const target = button.target;
    const name = target?.type === 'script' ? target.name : selectionName(stage, target);
    return button.label || name || 'Button';
}

export function findProfile(fixtureTypes: FixtureProfile[], type: string): FixtureProfile {
    return fixtureTypes.find((profile) => profile.type === type)!;
}

/// Moving heads also have the RGB controls, which pick their closest color wheel slot
export function controlKinds(profile: FixtureProfile): ControlKind[] {
    if (profile.kind === 'movingHead') return ['rgb', 'movingHead'];
    return profile.presets ? [profile.kind, 'preset'] : [profile.kind];
}

export function switchCount(profile: FixtureProfile): number {
    return profile.channels.filter((channel) => channel.type === 'switch').length;
}

// MARK: Store
/// The opened stage document, shared by all pages and kept in sync with the app
export const $document = signal<StageDocument | null>(null);

export function initStageStore(ipc: Ipc) {
    ipc.request('getStage').then((document) => ($document.value = document as StageDocument));
    // Updates that arrive before the document has loaded are already part of it
    ipc.on('setStage', ({ stage }: any) => {
        if ($document.value) $document.value = { ...$document.value, stage };
    });
    ipc.on('stageDirty', ({ path, dirty }: any) => {
        if ($document.value && $document.value.path === path) $document.value = { ...$document.value, dirty };
    });
    ipc.on('stageOpened', ({ path, stage }: any) => {
        if ($document.value) $document.value = { ...$document.value, path, stage, dirty: false };
    });

    ipc.request('getScripts').then((scripts) => ($scripts.value = scripts as ScriptsState));
    ipc.on('scriptsChanged', ({ scripts, folders }: any) => ($scripts.value = { ...$scripts.value, scripts, folders }));
    ipc.on('scriptsRunning', ({ running, errors }: any) => ($scripts.value = { ...$scripts.value, running, errors }));
}

// MARK: DMX output
export const $dmxLive = signal(false);

/// Run the DMX output while a page is shown, returns the fixture outputs for the visualization
export function useDmxOutput(ipc: Ipc, freeze: boolean): Record<number, FixtureOutput> {
    const [outputs, setOutputs] = useState<Record<number, FixtureOutput>>({});
    useEffect(() => {
        const listener = ipc.on('fixtureOutputs', ({ outputs }: any) => setOutputs(outputs));
        ipc.request('getState').then(({ state }: any) => setOutputs(state.fixtureOutputs));
        ipc.send('start', { freeze });
        $dmxLive.value = true;
        return () => {
            listener.remove();
            ipc.send('stop');
            $dmxLive.value = false;
        };
    }, []);
    return outputs;
}

// MARK: Scripts
export interface ScriptsState {
    /// Source of each script by name
    scripts: Record<string, string>;
    folders: string[];
    running: string[];
    /// Error of each script that stopped by an error
    errors: Record<string, string>;
}

export const $scripts = signal<ScriptsState>({ scripts: {}, folders: [], running: [], errors: {} });

export const scriptFolder = (path: string) => path.slice(0, Math.max(0, path.lastIndexOf('/')));
export const scriptLabel = (path: string) => path.slice(path.lastIndexOf('/') + 1);

export function scriptTree(names: string[], folders: string[]) {
    const directories = new Set<string>();
    const addDirectory = (path: string) => {
        const parts = path.split('/');
        for (let i = 1; i <= parts.length; i++) {
            const parent = parts.slice(0, i).join('/');
            if (parent) directories.add(parent);
        }
    };
    folders.forEach(addDirectory);
    names.forEach((name) => addDirectory(scriptFolder(name)));
    type Entry = { path: string; label: string; depth: number; folder: boolean };
    const children = (parent: string, depth: number): Entry[] => [
        ...[...directories]
            .filter((path) => scriptFolder(path) === parent)
            .sort()
            .flatMap((path) => [{ path, label: scriptLabel(path), depth, folder: true }, ...children(path, depth + 1)]),
        ...names
            .filter((path) => scriptFolder(path) === parent)
            .sort()
            .map((path) => ({ path, label: scriptLabel(path), depth, folder: false })),
    ];
    return children('', 1);
}

export function toggleScript(ipc: Ipc, name: string) {
    ipc.send($scripts.value.running.includes(name) ? 'stopScript' : 'startScript', { name });
}

export function updateStage(ipc: Ipc, stage: Stage) {
    $document.value = { ...$document.value!, stage, dirty: true };
    ipc.send('setStage', { stage });
}

let pendingSave: Promise<boolean> | null = null;

export async function saveStage(ipc: Ipc): Promise<boolean> {
    if (pendingSave && !(await pendingSave)) return false;
    if (!$document.value?.dirty) return true;
    pendingSave = (async () => {
        try {
            const { error } = (await ipc.request('saveStage')) as { error: string | null };
            if (error) throw new Error(error);
            return true;
        } catch (error) {
            window.alert(`Can't save stage: ${error instanceof Error ? error.message : error}`);
            return false;
        } finally {
            pendingSave = null;
        }
    })();
    return pendingSave;
}

export function fileName(path: string): string {
    return path.split(/[\\/]/).pop()!;
}
