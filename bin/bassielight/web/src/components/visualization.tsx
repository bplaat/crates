/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { mdiFlash, mdiLightbulbFluorescentTube, mdiPowerSocketDe, mdiSpotlightBeam, mdiWeatherFog } from '@mdi/js';
import type { ReadonlySignal } from '@preact/signals';
import type { ComponentChildren } from 'preact';
import { useLayoutEffect, useRef, useState } from 'preact/hooks';
import { boundsBox, pointBox, snapDrag, type Box, type Guide } from '../snap.ts';
import {
    buttonLabel,
    findProfile,
    highlightedFixtureIds,
    switchCount,
    type Button,
    type FixtureOutput,
    type FixtureProfile,
    type Selection,
    type Stage,
} from '../stage.ts';
import { colorToHex } from '../utils.ts';
import { GoboPattern } from './gobo.tsx';
import './visualization.css';

/// Snap distance in screen pixels
const SNAP_DISTANCE = 8;
const SWITCHES_PER_ROW = 4;
/// Switch packs, strobes, hazers and tubes are drawn as the same easy to see and click block
const BLOCK_SIZE: Size = { width: 69, height: 37 };

/// Pars with a front this close to square are round
const ROUND_ASPECT_TOLERANCE = 0.1;

/// Round pars are drawn at real size, other fixtures as a block
function bodySize(profile: FixtureProfile): Size {
    const [width, height] = profile.size;
    const diameter = Math.max(width, height);
    const isRound = profile.kind === 'rgb' && Math.abs(width - height) <= diameter * ROUND_ASPECT_TOLERANCE;
    return isRound ? { width: diameter / 10, height: diameter / 10 } : BLOCK_SIZE;
}

function buttonBox(button: Button): Box {
    return {
        minX: button.x - button.width / 2,
        minY: button.y - button.height / 2,
        maxX: button.x + button.width / 2,
        maxY: button.y + button.height / 2,
    };
}

/// Room and fixtures are drawn at real size in centimeters, the room scales to fit, while labels,
/// padding and strokes keep a constant on screen size. In the editor fixtures, groups and buttons
/// can be dragged and buttons select themselves, elsewhere the page handles button presses.
export function Visualization({
    stage,
    fixtureTypes,
    outputs,
    mode,
    selection,
    onSelect,
    onFixturesMove,
    onButtonMove,
    onButtonPress,
    isButtonActive,
}: {
    stage: Stage;
    fixtureTypes: FixtureProfile[];
    /// Read here so only the visualization re-renders on every DMX frame
    outputs?: ReadonlySignal<Record<number, FixtureOutput>>;
    mode?: string;
    selection: Selection;
    onSelect: (selection: Selection) => void;
    onFixturesMove?: (moves: { id: number; x: number; y: number }[]) => void;
    onButtonMove?: (id: number, x: number, y: number) => void;
    onButtonPress?: (button: Button) => void;
    isButtonActive?: (button: Button) => boolean;
}) {
    const svgRef = useRef<SVGSVGElement>(null);
    const [pixelsPerCm, setPixelsPerCm] = useState(1);
    const [drag, setDrag] = useState<{
        fixtureIds: number[];
        buttonId: number | null;
        start: { x: number; y: number };
        delta: { x: number; y: number };
        guides: Guide[];
    } | null>(null);
    const fixtureOutputs = outputs?.value;
    const isEditor = onFixturesMove !== undefined;
    const { width, height } = stage.room;
    const margin = Math.max(width, height) * 0.08;
    const px = (pixels: number) => pixels / pixelsPerCm;

    // Track the screen scale for the constant size labels and padding
    useLayoutEffect(() => {
        const svg = svgRef.current!;
        const update = () => {
            const scale = svg.getScreenCTM()?.a;
            if (scale && Math.abs(scale - pixelsPerCm) > 0.001) setPixelsPerCm(scale);
        };
        update();
        const observer = new ResizeObserver(update);
        observer.observe(svg);
        return () => observer.disconnect();
    }, [width, height, pixelsPerCm]);

    const moved = <T extends { x: number; y: number }>(item: T, isMoving: boolean): T =>
        drag && isMoving ? { ...item, x: item.x + drag.delta.x, y: item.y + drag.delta.y } : item;
    const fixtures = stage.fixtures.map((f) => moved(f, drag?.fixtureIds.includes(f.id) ?? false));
    const buttons = stage.buttons.map((b) => moved(b, drag?.buttonId === b.id));

    const groupBoxes = (items: typeof fixtures) =>
        stage.groups
            .filter((group) => !group.hide_outline)
            .map((group) => {
                const members = items.filter((f) => group.fixtures.includes(f.id));
                return members.length > 0 ? { group, box: boundsBox(members) } : null;
            })
            .filter((bounds) => bounds !== null);
    const groups = groupBoxes(fixtures)
        .map(({ group, box }, _, all) => {
            // Pad groups more for each group nested inside them so outlines and labels don't overlap
            const depth = all.filter(
                (other) =>
                    other.group !== group &&
                    other.box.minX >= box.minX &&
                    other.box.minY >= box.minY &&
                    other.box.maxX <= box.maxX &&
                    other.box.maxY <= box.maxY,
            ).length;
            const padding = px(28 + depth * 14);
            return {
                group,
                x: box.minX - padding,
                y: box.minY - padding,
                width: box.maxX - box.minX + padding * 2,
                // Extra room at the bottom for the fixture labels
                height: box.maxY - box.minY + padding * 2 + px(12),
            };
        })
        // Draw smaller groups on top so they stay clickable
        .sort((a, b) => b.width * b.height - a.width * a.height);

    const highlightedIds = highlightedFixtureIds(stage, selection);

    const toRoom = (event: PointerEvent) =>
        new DOMPoint(event.clientX, event.clientY).matrixTransform(svgRef.current!.getScreenCTM()!.inverse());

    // Editor only: drag fixtures, groups or buttons, snapping edges and centers unless Alt is held
    const startDrag = (event: PointerEvent, fixtureIds: number[], buttonId: number | null = null) => {
        if (!isEditor || (fixtureIds.length === 0 && buttonId === null)) return;
        svgRef.current!.setPointerCapture(event.pointerId);
        setDrag({ fixtureIds, buttonId, start: toRoom(event), delta: { x: 0, y: 0 }, guides: [] });
    };
    const moveDrag = (event: PointerEvent) => {
        if (!drag) return;
        const point = toRoom(event);
        const isMoving = (id: number) => drag.fixtureIds.includes(id);
        const moving = stage.fixtures.filter((f) => isMoving(f.id));
        const button = stage.buttons.find((b) => b.id === drag.buttonId);
        const targets = [
            ...stage.fixtures.filter((f) => !isMoving(f.id)).map((f) => pointBox(f.x, f.y)),
            ...groupBoxes(stage.fixtures)
                .filter(({ group }) => !group.fixtures.some(isMoving))
                .map(({ box }) => box),
            ...stage.buttons.filter((b) => b.id !== drag.buttonId).map(buttonBox),
        ];
        const { delta, guides } = snapDrag(
            // A button snaps with its rect, a group with its bounds and a fixture with its position
            [button ? buttonBox(button) : moving.length > 1 ? boundsBox(moving) : pointBox(moving[0].x, moving[0].y)],
            targets,
            stage.room,
            { x: Math.round(point.x - drag.start.x), y: Math.round(point.y - drag.start.y) },
            event.altKey ? null : px(SNAP_DISTANCE),
        );
        setDrag({ ...drag, delta, guides });
    };
    const endDrag = () => {
        if (!drag) return;
        if (drag.delta.x !== 0 || drag.delta.y !== 0) {
            const button = buttons.find((b) => b.id === drag.buttonId);
            if (button) onButtonMove?.(button.id, button.x, button.y);
            else
                onFixturesMove!(
                    fixtures.filter((f) => drag.fixtureIds.includes(f.id)).map(({ id, x, y }) => ({ id, x, y })),
                );
        }
        setDrag(null);
    };

    return (
        <svg
            ref={svgRef}
            class={`visualization ${mode ? `is-mode-${mode}` : ''}`}
            viewBox={`${-margin} ${-margin} ${width + margin * 2} ${height + margin * 2}`}
            style={{ fontSize: px(12) }}
            onPointerMove={moveDrag}
            onPointerUp={endDrag}
        >
            <defs>
                <linearGradient id="visualization-music" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0%" stop-color="#f43f5e" />
                    <stop offset="33%" stop-color="#eab308" />
                    <stop offset="66%" stop-color="#22c55e" />
                    <stop offset="100%" stop-color="#3b82f6" />
                    <animateTransform
                        attributeName="gradientTransform"
                        type="rotate"
                        from="0 0.5 0.5"
                        to="360 0.5 0.5"
                        dur="4s"
                        repeatCount="indefinite"
                    />
                </linearGradient>
                <radialGradient id="visualization-haze">
                    <stop offset="40%" stop-color="#cbd5e1" />
                    <stop offset="100%" stop-color="#cbd5e1" stop-opacity="0" />
                </radialGradient>
            </defs>

            <rect
                class={`visualization-room ${selection === null ? 'is-selected' : ''}`}
                width={width}
                height={height}
                onPointerDown={() => onSelect(null)}
            />

            {groups.map(({ group, ...bounds }) => (
                <g
                    key={group.id}
                    class={`visualization-group ${selection?.type === 'group' && selection.id === group.id ? 'is-selected' : ''}`}
                    onPointerDown={(event) => {
                        onSelect({ type: 'group', id: group.id });
                        startDrag(event, group.fixtures);
                    }}
                >
                    <rect {...bounds} rx={px(8)} />
                    <text x={bounds.x} y={bounds.y - px(5)}>
                        {group.name}
                    </text>
                </g>
            ))}

            {buttons.map((button) => {
                const isSelected = isEditor
                    ? selection?.type === 'button' && selection.id === button.id
                    : (isButtonActive?.(button) ?? false);
                const box = buttonBox(button);
                return (
                    <g
                        key={button.id}
                        class={`visualization-button ${isSelected ? 'is-selected' : ''}`}
                        onPointerDown={(event) => {
                            event.stopPropagation();
                            if (isEditor) {
                                onSelect({ type: 'button', id: button.id });
                                startDrag(event, [], button.id);
                            } else {
                                onButtonPress?.(button);
                            }
                        }}
                    >
                        <rect
                            x={box.minX}
                            y={box.minY}
                            width={button.width}
                            height={button.height}
                            rx={Math.min(px(8), button.height / 2)}
                        />
                        <text x={button.x} y={button.y} text-anchor="middle" dominant-baseline="central">
                            {buttonLabel(stage, button)}
                        </text>
                    </g>
                );
            })}

            {fixtures.map((fixture) => {
                const profile = findProfile(fixtureTypes, fixture.type);
                const output = fixtureOutputs?.[fixture.id];
                const size = bodySize(profile);
                const isSelected = selection?.type === 'fixture' && selection.id === fixture.id;
                const isDimmed = highlightedIds !== null && !highlightedIds.includes(fixture.id);
                return (
                    <g
                        key={fixture.id}
                        class={`visualization-fixture ${isSelected ? 'is-selected' : ''} ${isDimmed ? 'is-dimmed' : ''}`}
                        transform={`translate(${fixture.x} ${fixture.y})`}
                        onPointerDown={(event) => {
                            event.stopPropagation();
                            onSelect({ type: 'fixture', id: fixture.id });
                            startDrag(event, [fixture.id]);
                        }}
                    >
                        {profile.kind === 'rgb' && (
                            <RgbShape color={output && 'rgb' in output ? output.rgb : undefined} size={size}>
                                <BlockIcon path={mdiLightbulbFluorescentTube} size={size} />
                            </RgbShape>
                        )}
                        {profile.kind === 'movingHead' && (
                            <MovingHeadShape
                                output={output && 'movingHead' in output ? output.movingHead : undefined}
                                size={size}
                            />
                        )}
                        {profile.kind === 'switch' && (
                            <SwitchShape
                                count={switchCount(profile)}
                                states={output && 'switch' in output ? output.switch : undefined}
                                size={size}
                            />
                        )}
                        {profile.kind === 'strobe' && (
                            <StrobeShape
                                output={output && 'strobe' in output ? output.strobe : undefined}
                                size={size}
                            />
                        )}
                        {profile.kind === 'haze' && (
                            <HazeShape output={output && 'haze' in output ? output.haze : undefined} size={size} />
                        )}
                        <text y={size.height / 2 + px(16)} text-anchor="middle">
                            {fixture.name}
                        </text>
                    </g>
                );
            })}

            {drag?.guides.map((guide) => (
                <line
                    key={`${guide.axis}-${guide.value}`}
                    class="visualization-guide"
                    {...(guide.axis === 'x'
                        ? { x1: guide.value, x2: guide.value, y1: guide.from, y2: guide.to }
                        : { x1: guide.from, x2: guide.to, y1: guide.value, y2: guide.value })}
                />
            ))}
        </svg>
    );
}

type Size = { width: number; height: number };

/// Round pars are circles, other fixtures like tubes and moving heads a block with their icon. A
/// fixture running its own program has no known color.
function RgbShape({
    color,
    size,
    children,
}: {
    color: number | null | undefined;
    size: Size;
    children: ComponentChildren;
}) {
    const fill = color != null ? { fill: colorToHex(color) } : undefined;
    const bodyClass = `visualization-body is-rgb ${color === null ? 'is-program' : ''}`;
    const hasGlow = color != null && color !== 0;
    if (size.width === size.height) {
        return (
            <>
                {hasGlow && <circle class="visualization-glow" r={size.width * 0.875} style={fill} />}
                <circle class={bodyClass} r={size.width / 2} style={fill} />
            </>
        );
    }
    const glow = size.height * 0.4;
    return (
        <>
            {hasGlow && (
                <rect
                    class="visualization-glow"
                    {...blockRect({ width: size.width + glow * 2, height: size.height + glow * 2 })}
                    style={fill}
                />
            )}
            <rect class={bodyClass} {...blockRect(size)} style={fill} />
            {children}
        </>
    );
}

/// Moving heads show the gobo in their beam, or a spotlight for the open beam
function MovingHeadShape({ output, size }: { output?: { color: number | null; gobo: string | null }; size: Size }) {
    const goboSize = Math.min(size.width, size.height) * 0.8;
    return (
        <RgbShape color={output?.color} size={size}>
            {output?.gobo ? (
                <g transform={`scale(${goboSize / 24})`}>
                    <circle class="gobo-disc" r={11} />
                    <GoboPattern shape={output.gobo} />
                </g>
            ) : (
                <BlockIcon path={mdiSpotlightBeam} size={size} />
            )}
        </RgbShape>
    );
}

function blockRect(size: Size) {
    return {
        x: -size.width / 2,
        y: -size.height / 2,
        width: size.width,
        height: size.height,
        rx: Math.min(size.width, size.height) * 0.2,
    };
}

/// Monochrome icon in the middle of a block
function BlockIcon({ path, size }: { path: string; size: Size }) {
    const iconSize = Math.min(size.width, size.height) * 0.7;
    return (
        <path
            class="visualization-block-icon"
            d={path}
            transform={`translate(${-iconSize / 2} ${-iconSize / 2}) scale(${iconSize / 24})`}
        />
    );
}

/// Schuko socket icon on top with a dot for each switch below, green when the switch is on
function SwitchShape({ count, states, size }: { count: number; states?: boolean[]; size: Size }) {
    const columns = Math.min(count, SWITCHES_PER_ROW);
    const rows = Math.ceil(count / SWITCHES_PER_ROW);
    const iconSize = size.height * 0.45;
    const cell = Math.min(size.width / (columns + 1), (size.height * 0.4) / rows);
    const top = -size.height / 2 + size.height * 0.08;
    return (
        <>
            <rect class="visualization-body" {...blockRect(size)} />
            <path
                class="visualization-block-icon"
                d={mdiPowerSocketDe}
                transform={`translate(${-iconSize / 2} ${top}) scale(${iconSize / 24})`}
            />
            {Array.from({ length: count }, (_, index) => (
                <circle
                    key={index}
                    class={`visualization-switch ${states?.[index] ? 'is-on' : ''}`}
                    cx={((index % SWITCHES_PER_ROW) - (columns - 1) / 2) * cell}
                    cy={top + iconSize + (Math.floor(index / SWITCHES_PER_ROW) + 0.6) * cell}
                    r={cell * 0.3}
                />
            ))}
        </>
    );
}

/// The Titan Strobe flashes 1 to 15 times per second
function StrobeShape({ output, size }: { output?: { intensity: number; speed: number }; size: Size }) {
    const isFlashing = output !== undefined && output.intensity > 0;
    return (
        <>
            <rect class="visualization-body" {...blockRect(size)} />
            {isFlashing && (
                <rect
                    class="visualization-flash"
                    {...blockRect(size)}
                    style={{
                        '--flash-opacity': output.intensity,
                        animationDuration: `${1 / (1 + output.speed * 14)}s`,
                    }}
                />
            )}
            <BlockIcon path={mdiFlash} size={size} />
        </>
    );
}

/// A cloud around the hazer that grows with its volume, it drifts faster with the fan
function HazeShape({ output, size }: { output?: { on: boolean; volume: number; fan: number }; size: Size }) {
    return (
        <>
            {output?.on && (
                <rect
                    class="visualization-haze"
                    {...blockRect({
                        width: size.width + size.height * (1.2 + output.volume * 1.2),
                        height: size.height * (2.2 + output.volume * 1.2),
                    })}
                    style={{ animationDuration: `${4 - output.fan * 3}s` }}
                />
            )}
            <rect class="visualization-body" {...blockRect(size)} />
            <BlockIcon path={mdiWeatherFog} size={size} />
        </>
    );
}
