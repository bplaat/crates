/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { useContext, useEffect, useRef, useState } from 'preact/hooks';
import { IpcContext } from '../app.tsx';
import { DipSwitch } from '../components/dipswitch.tsx';
import {
    CloseIcon,
    DeleteIcon,
    GestureTapButtonIcon,
    GroupIcon,
    KindIcon,
    LightbulbIcon,
} from '../components/icons.tsx';
import { Visualization } from '../components/visualization.tsx';
import {
    $document,
    $scripts,
    buttonLabel,
    findProfile,
    nextId,
    pasteOffset,
    switchCount,
    updateStage,
    type Button,
    type ButtonTarget,
    type Fixture,
    type Group,
    type Selection,
    type Stage,
    type Target,
} from '../stage.ts';

const ROOM_MIN = 100;
const ROOM_MAX = 10000;
const BUTTON_MIN = 10;

function TextField({
    label,
    value,
    placeholder,
    onChange,
}: {
    label: string;
    value: string;
    placeholder?: string;
    onChange: (value: string) => void;
}) {
    return (
        <label class="field">
            <span class="field-label">{label}</span>
            <input
                class="input"
                type="text"
                value={value}
                placeholder={placeholder}
                onInput={(e) => onChange(e.currentTarget.value)}
            />
        </label>
    );
}

function NumberField({
    label,
    value,
    min,
    max,
    onChange,
    onCommit,
    children,
}: {
    label: string;
    value: number;
    min: number;
    max: number;
    onChange: (value: number) => void;
    /// Called when editing is done, on enter or blur
    onCommit?: () => void;
    children?: ComponentChildren;
}) {
    const input = (
        <input
            class="input"
            type="number"
            min={min}
            max={max}
            value={value}
            // Only commit valid values while typing, restore the current value on blur
            onInput={(e) => {
                const input = e.currentTarget.value;
                const newValue = Number(input);
                if (input !== '' && Number.isInteger(newValue) && newValue >= min && newValue <= max) {
                    onChange(newValue);
                }
            }}
            onKeyDown={(e) => {
                if (e.key === 'Enter') e.currentTarget.blur();
            }}
            onBlur={(e) => {
                e.currentTarget.value = String(value);
                onCommit?.();
            }}
        />
    );
    return (
        <label class="field">
            <span class="field-label">{label}</span>
            {children ? (
                <div class="field-row">
                    {input}
                    {children}
                </div>
            ) : (
                input
            )}
        </label>
    );
}

function SidebarHeader({ onClose, children }: { onClose: () => void; children: ComponentChildren }) {
    return (
        <div class="sidebar-header">
            <h2 class="title has-icon">{children}</h2>
            <button class="icon-button" title="Back to room" onClick={onClose}>
                <CloseIcon />
            </button>
        </div>
    );
}

/// Button target values are `type:id`, or `script:name`
const targetValue = (target: ButtonTarget) =>
    target.type === 'script' ? `script:${target.name}` : `${target.type}:${target.id}`;

function parseTarget(value: string): ButtonTarget | null {
    const separator = value.indexOf(':');
    const type = value.slice(0, separator);
    const rest = value.slice(separator + 1);
    if (type === 'script') return { type, name: rest };
    if (type === 'fixture' || type === 'group') return { type, id: Number(rest) };
    return null;
}

/// Copied fixtures, with the group they were copied as, kept while switching pages
let clipboard: { fixtures: Fixture[]; group?: Group } | null = null;

export function EditorPage() {
    const ipc = useContext(IpcContext)!;
    const [selection, setSelection] = useState<Selection>(null);
    const shortcuts = useRef<{ copy: () => void; paste: () => void; remove: () => void }>(null);
    const path = $document.value?.path;

    useEffect(() => {
        document.title = 'BassieLight - Editor';
        const onKeyDown = (event: KeyboardEvent) => {
            if ((event.target as Element).closest('input, select, textarea')) return;
            const action =
                event.metaKey || event.ctrlKey
                    ? { c: shortcuts.current?.copy, v: shortcuts.current?.paste }[event.key]
                    : { Delete: shortcuts.current?.remove, Backspace: shortcuts.current?.remove }[event.key];
            if (action) {
                event.preventDefault();
                action();
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, []);
    useEffect(() => setSelection(null), [path]);

    if (!$document.value) return null;
    const { stage, fixtureTypes, dmxLength } = $document.value;
    const { room } = stage;

    const update = (patch: Partial<Stage>) => updateStage(ipc, { ...stage, ...patch });
    // Fixtures and buttons outside a smaller room are only moved in when done editing
    const clampToRoom = () => {
        const clamp = <T extends { x: number; y: number }>(item: T): T => ({
            ...item,
            x: Math.min(item.x, room.width),
            y: Math.min(item.y, room.height),
        });
        update({ fixtures: stage.fixtures.map(clamp), buttons: stage.buttons.map(clamp) });
    };
    const updateFixture = (id: number, patch: Partial<Fixture>) =>
        update({ fixtures: stage.fixtures.map((f) => (f.id === id ? { ...f, ...patch } : f)) });
    const updateGroup = (id: number, patch: Partial<Group>) =>
        update({ groups: stage.groups.map((g) => (g.id === id ? { ...g, ...patch } : g)) });
    const toggleMember = (group: Group, fixtureId: number) =>
        updateGroup(group.id, {
            fixtures: group.fixtures.includes(fixtureId)
                ? group.fixtures.filter((id) => id !== fixtureId)
                : [...group.fixtures, fixtureId],
        });
    const profileOf = (type: string) => findProfile(fixtureTypes, type);
    const maxAddr = (type: string) => dmxLength - profileOf(type).channels.length + 1;

    const addFixture = () => {
        const id = nextId(stage.fixtures);
        const type = fixtureTypes[0].type;
        const addr = Math.max(1, ...stage.fixtures.map((f) => f.addr + profileOf(f.type).channels.length));
        update({
            fixtures: [
                ...stage.fixtures,
                {
                    id,
                    name: `Fixture ${id}`,
                    type,
                    addr: Math.min(addr, maxAddr(type)),
                    x: Math.round(room.width / 2),
                    y: Math.round(room.height / 2),
                },
            ],
        });
        setSelection({ type: 'fixture', id });
    };
    // Buttons of a deleted fixture or group stay, without a target
    const untargetButtons = (target: Target) =>
        stage.buttons.map((b) =>
            b.target && b.target.type !== 'script' && b.target.type === target.type && b.target.id === target.id
                ? { ...b, target: null }
                : b,
        );
    const deleteFixture = (id: number) => {
        update({
            fixtures: stage.fixtures.filter((f) => f.id !== id),
            groups: stage.groups.map((g) => ({ ...g, fixtures: g.fixtures.filter((fixtureId) => fixtureId !== id) })),
            buttons: untargetButtons({ type: 'fixture', id }),
        });
        setSelection(null);
    };
    const addGroup = () => {
        const id = nextId(stage.groups);
        update({ groups: [...stage.groups, { id, name: `Group ${id}`, fixtures: [] }] });
        setSelection({ type: 'group', id });
    };
    const deleteGroup = (id: number) => {
        update({ groups: stage.groups.filter((g) => g.id !== id), buttons: untargetButtons({ type: 'group', id }) });
        setSelection(null);
    };
    const updateButton = (id: number, patch: Partial<Button>) =>
        update({ buttons: stage.buttons.map((b) => (b.id === id ? { ...b, ...patch } : b)) });
    const addButton = () => {
        const id = nextId(stage.buttons);
        const width = Math.round(Math.max(room.width, room.height) / 8);
        update({
            buttons: [
                ...stage.buttons,
                {
                    id,
                    label: '',
                    x: Math.round(room.width / 2),
                    y: Math.round(room.height / 2),
                    width,
                    height: Math.round(width / 2.5),
                    // Start as a shortcut for the selected fixture or group
                    target: selection?.type === 'fixture' || selection?.type === 'group' ? selection : null,
                },
            ],
        });
        setSelection({ type: 'button', id });
    };
    const deleteButton = (id: number) => {
        update({ buttons: stage.buttons.filter((b) => b.id !== id) });
        setSelection(null);
    };

    // Copies keep their addresses, pasting offsets them a bit so they don't hide the originals
    shortcuts.current = {
        copy: () => {
            if (selection?.type === 'fixture') {
                const fixture = stage.fixtures.find((f) => f.id === selection.id);
                if (fixture) clipboard = { fixtures: [fixture] };
            }
            if (selection?.type === 'group') {
                const group = stage.groups.find((g) => g.id === selection.id);
                if (group) clipboard = { fixtures: stage.fixtures.filter((f) => group.fixtures.includes(f.id)), group };
            }
        },
        remove: () => {
            if (selection?.type === 'fixture') deleteFixture(selection.id);
            if (selection?.type === 'group') deleteGroup(selection.id);
            if (selection?.type === 'button') deleteButton(selection.id);
        },
        paste: () => {
            if (!clipboard || clipboard.fixtures.length === 0) return;
            const offset = pasteOffset(room);
            const firstId = nextId(stage.fixtures);
            const ids = new Map(clipboard.fixtures.map((f, index) => [f.id, firstId + index]));
            const fixtures = clipboard.fixtures.map((f) => ({
                ...f,
                id: ids.get(f.id)!,
                x: Math.min(f.x + offset, room.width),
                y: Math.min(f.y + offset, room.height),
            }));
            const group = clipboard.group && {
                ...clipboard.group,
                id: nextId(stage.groups),
                fixtures: clipboard.group.fixtures.filter((id) => ids.has(id)).map((id) => ids.get(id)!),
            };
            update({
                fixtures: [...stage.fixtures, ...fixtures],
                groups: group ? [...stage.groups, group] : stage.groups,
            });
            clipboard = { fixtures, group };
            setSelection(group ? { type: 'group', id: group.id } : { type: 'fixture', id: fixtures[0].id });
        },
    };

    const fixture = selection?.type === 'fixture' ? stage.fixtures.find((f) => f.id === selection.id) : undefined;
    const profile = fixture && profileOf(fixture.type);
    const group = selection?.type === 'group' ? stage.groups.find((g) => g.id === selection.id) : undefined;
    const button = selection?.type === 'button' ? stage.buttons.find((b) => b.id === selection.id) : undefined;

    return (
        <>
            <div class="main">
                <Visualization
                    stage={stage}
                    fixtureTypes={fixtureTypes}
                    selection={selection}
                    onSelect={setSelection}
                    onFixturesMove={(moves) =>
                        update({
                            fixtures: stage.fixtures.map((f) => {
                                const move = moves.find((m) => m.id === f.id);
                                return move ? { ...f, x: move.x, y: move.y } : f;
                            }),
                        })
                    }
                    onButtonMove={(id, x, y) => updateButton(id, { x, y })}
                />

                <div class="buttons is-centered">
                    <button class="button is-expanded" onClick={addFixture}>
                        <LightbulbIcon />
                        Add fixture
                    </button>
                    <button class="button is-expanded" onClick={addGroup}>
                        <GroupIcon />
                        Add group
                    </button>
                    <button class="button is-expanded" onClick={addButton}>
                        <GestureTapButtonIcon />
                        Add button
                    </button>
                </div>
            </div>

            <div class="sidebar">
                {fixture && (
                    <>
                        <SidebarHeader onClose={() => setSelection(null)}>
                            <KindIcon kind={profile!.kind} />
                            Fixture
                        </SidebarHeader>
                        <TextField
                            label="Name"
                            value={fixture.name}
                            onChange={(name) => updateFixture(fixture.id, { name })}
                        />
                        <label class="field">
                            <span class="field-label">Type</span>
                            <select
                                class="input"
                                value={fixture.type}
                                onChange={(e) => {
                                    const type = e.currentTarget.value;
                                    const count = switchCount(profileOf(type));
                                    updateFixture(fixture.id, {
                                        type,
                                        addr: Math.min(fixture.addr, maxAddr(type)),
                                        switches:
                                            count > 0
                                                ? Array.from({ length: count }, (_, i) => fixture.switches?.[i] ?? '')
                                                : undefined,
                                    });
                                }}
                            >
                                {fixtureTypes.map(({ type, name }) => (
                                    <option key={type} value={type}>
                                        {name}
                                    </option>
                                ))}
                            </select>
                        </label>
                        <NumberField
                            label="DMX address"
                            value={fixture.addr}
                            min={1}
                            max={maxAddr(fixture.type)}
                            onChange={(addr) => updateFixture(fixture.id, { addr })}
                        >
                            <DipSwitch
                                value={fixture.addr}
                                onChange={(addr) => {
                                    if (addr >= 1 && addr <= maxAddr(fixture.type)) updateFixture(fixture.id, { addr });
                                }}
                            />
                        </NumberField>
                        {fixture.switches && (
                            <>
                                <h2 class="title">Switches</h2>
                                {fixture.switches.map((label, index) => (
                                    <TextField
                                        key={index}
                                        label={`Switch ${index + 1}`}
                                        value={label}
                                        onChange={(value) =>
                                            updateFixture(fixture.id, {
                                                switches: fixture.switches!.map((l, i) => (i === index ? value : l)),
                                            })
                                        }
                                    />
                                ))}
                            </>
                        )}

                        {stage.groups.length > 0 && (
                            <>
                                <h2 class="title">Groups</h2>
                                {stage.groups.map((group) => (
                                    <label key={group.id} class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={group.fixtures.includes(fixture.id)}
                                            onChange={() => toggleMember(group, fixture.id)}
                                        />
                                        {group.name}
                                    </label>
                                ))}
                            </>
                        )}

                        <div class="buttons is-centered">
                            <button class="button is-expanded is-danger" onClick={() => deleteFixture(fixture.id)}>
                                <DeleteIcon />
                                Delete fixture
                            </button>
                        </div>
                    </>
                )}

                {group && (
                    <>
                        <SidebarHeader onClose={() => setSelection(null)}>
                            <GroupIcon />
                            Group
                        </SidebarHeader>
                        <TextField
                            label="Name"
                            value={group.name}
                            onChange={(name) => updateGroup(group.id, { name })}
                        />
                        <label class="checkbox">
                            <input
                                type="checkbox"
                                checked={!group.hide_outline}
                                onChange={() => updateGroup(group.id, { hide_outline: !group.hide_outline })}
                            />
                            Show outline
                        </label>

                        <h2 class="title">Fixtures</h2>
                        {stage.fixtures.length === 0 && <p class="block">No fixtures yet</p>}
                        {stage.fixtures.map((fixture) => (
                            <label key={fixture.id} class="checkbox">
                                <input
                                    type="checkbox"
                                    checked={group.fixtures.includes(fixture.id)}
                                    onChange={() => toggleMember(group, fixture.id)}
                                />
                                {fixture.name}
                            </label>
                        ))}

                        <div class="buttons is-centered">
                            <button class="button is-expanded is-danger" onClick={() => deleteGroup(group.id)}>
                                <DeleteIcon />
                                Delete group
                            </button>
                        </div>
                    </>
                )}

                {button && (
                    <>
                        <SidebarHeader onClose={() => setSelection(null)}>
                            <GestureTapButtonIcon />
                            Button
                        </SidebarHeader>
                        <label class="field">
                            <span class="field-label">Selects</span>
                            <select
                                class="input"
                                value={button.target ? targetValue(button.target) : ''}
                                onChange={(e) =>
                                    updateButton(button.id, { target: parseTarget(e.currentTarget.value) })
                                }
                            >
                                <option value="">Nothing</option>
                                <optgroup label="Scripts">
                                    {Object.keys($scripts.value.scripts).map((name) => (
                                        <option key={name} value={`script:${name}`}>
                                            {name}
                                        </option>
                                    ))}
                                </optgroup>
                                <optgroup label="Groups">
                                    {stage.groups.map((group) => (
                                        <option key={group.id} value={`group:${group.id}`}>
                                            {group.name}
                                        </option>
                                    ))}
                                </optgroup>
                                <optgroup label="Fixtures">
                                    {stage.fixtures.map((fixture) => (
                                        <option key={fixture.id} value={`fixture:${fixture.id}`}>
                                            {fixture.name}
                                        </option>
                                    ))}
                                </optgroup>
                            </select>
                        </label>
                        <TextField
                            label="Label"
                            value={button.label}
                            placeholder={buttonLabel(stage, { ...button, label: '' })}
                            onChange={(label) => updateButton(button.id, { label })}
                        />
                        <div class="fields">
                            <NumberField
                                label="Width (cm)"
                                value={button.width}
                                min={BUTTON_MIN}
                                max={room.width}
                                onChange={(width) => updateButton(button.id, { width })}
                            />
                            <NumberField
                                label="Height (cm)"
                                value={button.height}
                                min={BUTTON_MIN}
                                max={room.height}
                                onChange={(height) => updateButton(button.id, { height })}
                            />
                        </div>

                        <div class="buttons is-centered">
                            <button class="button is-expanded is-danger" onClick={() => deleteButton(button.id)}>
                                <DeleteIcon />
                                Delete button
                            </button>
                        </div>
                    </>
                )}

                {!fixture && !group && !button && (
                    <>
                        <h2 class="title">Room</h2>
                        <div class="fields">
                            <NumberField
                                label="Width (cm)"
                                value={room.width}
                                min={ROOM_MIN}
                                max={ROOM_MAX}
                                onChange={(width) => update({ room: { ...room, width } })}
                                onCommit={clampToRoom}
                            />
                            <NumberField
                                label="Height (cm)"
                                value={room.height}
                                min={ROOM_MIN}
                                max={ROOM_MAX}
                                onChange={(height) => update({ room: { ...room, height } })}
                                onCommit={clampToRoom}
                            />
                        </div>

                        <h2 class="title">Fixtures</h2>
                        <div class="list">
                            {stage.fixtures.length === 0 && <p>No fixtures yet</p>}
                            {stage.fixtures.map((fixture) => (
                                <button
                                    key={fixture.id}
                                    class="button"
                                    onClick={() => setSelection({ type: 'fixture', id: fixture.id })}
                                >
                                    <KindIcon kind={profileOf(fixture.type).kind} />
                                    {fixture.name}
                                    <span class="list-meta">DMX {fixture.addr}</span>
                                </button>
                            ))}
                        </div>

                        <h2 class="title">Groups</h2>
                        <div class="list">
                            {stage.groups.length === 0 && <p>No groups yet</p>}
                            {stage.groups.map((group) => (
                                <button
                                    key={group.id}
                                    class="button"
                                    onClick={() => setSelection({ type: 'group', id: group.id })}
                                >
                                    <GroupIcon />
                                    {group.name}
                                    <span class="list-meta">{group.fixtures.length} fixtures</span>
                                </button>
                            ))}
                        </div>

                        <h2 class="title">Buttons</h2>
                        <div class="list">
                            {stage.buttons.length === 0 && <p>No buttons yet</p>}
                            {stage.buttons.map((button) => (
                                <button
                                    key={button.id}
                                    class="button"
                                    onClick={() => setSelection({ type: 'button', id: button.id })}
                                >
                                    <GestureTapButtonIcon />
                                    {buttonLabel(stage, button)}
                                </button>
                            ))}
                        </div>
                    </>
                )}
            </div>
        </>
    );
}
