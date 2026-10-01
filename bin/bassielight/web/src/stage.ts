/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import type { Ipc } from './ipc.ts';

export type Channel =
    | { type: 'red' | 'green' | 'blue' | 'dimmer' | 'switch' | 'speed' | 'preset' | 'unused' }
    | { type: 'music'; value: number };

export type FixtureKind = 'rgb' | 'switch' | 'strobe';

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
}

export type FixtureOutput =
    { rgb: number | null } | { switch: boolean[] } | { strobe: { intensity: number; speed: number } };

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
    target: Target | null;
}

export interface Stage {
    room: { width: number; height: number };
    fixtures: Fixture[];
    groups: Group[];
    buttons: Button[];
}

export interface StageDocument {
    path: string;
    stage: Stage;
    fixtureTypes: FixtureProfile[];
    dmxLength: number;
}

export type Target = { type: 'fixture' | 'group'; id: number };
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
    return target ? selectedFixtureIds(stage, target) : null;
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
    return button.label || selectionName(stage, button.target) || 'Button';
}

export function findProfile(fixtureTypes: FixtureProfile[], type: string): FixtureProfile {
    return fixtureTypes.find((profile) => profile.type === type)!;
}

export function controlKinds(profile: FixtureProfile): ControlKind[] {
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
    ipc.on('stageOpened', ({ path, stage }: any) => {
        if ($document.value) $document.value = { ...$document.value, path, stage };
    });
}

export function updateStage(ipc: Ipc, stage: Stage) {
    $document.value = { ...$document.value!, stage };
    ipc.send('setStage', { stage });
}

export function fileName(path: string): string {
    return path.split(/[\\/]/).pop()!;
}
