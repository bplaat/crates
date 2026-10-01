/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import './gobo.css';

// Gobo patterns are drawn in a 24 by 24 box around the center, the light is white
const around = (count: number, radius: number, offset = 0) =>
    Array.from({ length: count }, (_, index) => {
        const angle = offset + (index / count) * Math.PI * 2;
        return { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius, angle };
    });
const dots = (points: { x: number; y: number }[], r: number) =>
    points.map((p, index) => <circle key={index} cx={p.x} cy={p.y} r={r} />);
const ring = (r: number, width: number) => <circle r={r} fill="none" stroke-width={width} />;
const rotated = (count: number, child: (index: number) => ComponentChildren) =>
    Array.from({ length: count }, (_, index) => (
        <g key={index} transform={`rotate(${(index * 360) / count})`}>
            {child(index)}
        </g>
    ));
const spiral = (turns: number, radius: number) => {
    const steps = 48;
    const points = Array.from({ length: steps + 1 }, (_, index) => {
        const t = index / steps;
        const angle = t * turns * Math.PI * 2;
        return `${Math.cos(angle) * radius * t},${Math.sin(angle) * radius * t}`;
    });
    return `M${points.join(' L')}`;
};

const SHAPES: Record<string, () => ComponentChildren> = {
    open: () => <circle r={8.5} />,
    ringLarge: () => ring(7, 2.5),
    ringMedium: () => ring(4.8, 2.2),
    ringSmall: () => ring(2.6, 1.6),
    dot: () => <circle r={1.4} />,
    dotLine: () =>
        dots(
            [-2, -1, 0, 1, 2].map((t) => ({ x: t * 3.2, y: -t * 1.6 })),
            1.1,
        ),
    arrows: () =>
        rotated(3, () => <path d="M0,-7.5 L2.6,-3.2 L0.9,-3.6 L0.9,-1.5 L-0.9,-1.5 L-0.9,-3.6 L-2.6,-3.2 Z" />),
    dotCloud: () =>
        dots(
            [
                [0, 0],
                [3, -1],
                [-3, 1],
                [1, 3.5],
                [-1.5, -3.5],
                [5, 2.5],
                [-5, -2],
                [4, -4.5],
                [-4, 4.5],
                [1, -6.5],
                [-0.5, 6.5],
                [6.5, -1],
                [-6.5, 0.5],
            ].map(([x, y]) => ({ x, y })),
            0.8,
        ),
    dotScatter: () =>
        dots(
            [
                [-3.5, -5],
                [3, -4],
                [-5.5, 1],
                [0, 0],
                [5, 2],
                [-2, 5.5],
                [3, 6],
            ].map(([x, y]) => ({ x, y })),
            1.6,
        ),
    wave: () => <path d="M-7.5,2 C-4.5,-5 -1.5,-5 0,0 S4.5,5 7.5,-2" fill="none" stroke-width={2.2} />,
    squares: () =>
        [-1, 1].flatMap((x) =>
            [-1, 1].map((y) => <rect key={`${x}${y}`} x={x * 3.4 - 2.4} y={y * 3.4 - 2.4} width={4.8} height={4.8} />),
        ),
    dotDisc: () => dots([{ x: 0, y: 0 }, ...around(6, 3), ...around(12, 6, Math.PI / 12)], 0.85),
    flower: () => (
        <>
            {around(4, 3.6, Math.PI / 4).map((p, index) => (
                <circle key={index} cx={p.x} cy={p.y} r={3} />
            ))}
            <circle r={2} />
        </>
    ),
    spiral: () => <path d={spiral(2, 7.5)} fill="none" stroke-width={1.8} />,
    dotRing: () => dots(around(10, 6.5), 1.1),
    cross: () => (
        <>
            <rect x={-1.3} y={-7.5} width={2.6} height={15} />
            <rect x={-7.5} y={-1.3} width={15} height={2.6} />
        </>
    ),
    shards: () =>
        rotated(4, (index) => (
            <path d={index % 2 ? 'M1.5,-7 L3.5,-2 L2,-2.5 L1,-1.5 Z' : 'M-1,-7.5 L2,-3.5 L0.5,-3.5 L-1.5,-1.5 Z'} />
        )),
    swirl: () => rotated(6, () => <path d="M0,0 C2,-2 2,-6 0,-8.5 C4,-6.5 4.5,-2.5 0,0 Z" />),
    petals: () => rotated(6, () => <ellipse cy={-4.5} rx={2} ry={3.8} />),
    circles: () => dots(around(4, 4, Math.PI / 4), 3.2),
    vortex: () => rotated(3, () => <path d={spiral(0.6, 8.5)} fill="none" stroke-width={2.4} />),
    starburst: () =>
        rotated(8, () =>
            dots(
                [2.5, 4.5, 6.5, 8.3].map((y, index) => ({ x: 0, y: -y, index })),
                0.7,
            ),
        ),
    ringGap: () => <path d="M5.2,-5.2 A7.4,7.4 0 1,0 7.4,0" fill="none" stroke-width={2.4} />,
    waves: () =>
        [-6, -3, 0, 3, 6].map((x) => (
            <path
                key={x}
                d={`M${x},-7 q1.5,1.75 0,3.5 q-1.5,1.75 0,3.5 q1.5,1.75 0,3.5 q-1.5,1.75 0,3.5`}
                fill="none"
                stroke-width={1.1}
            />
        )),
    rings: () =>
        around(3, 3, -Math.PI / 2).map((p, index) => (
            <circle key={index} cx={p.x} cy={p.y} r={4} fill="none" stroke-width={1.4} />
        )),
};

/// Light pattern of a gobo, `null` is the open beam
export function GoboPattern({ shape }: { shape: string | null }) {
    return <g class="gobo-pattern">{(SHAPES[shape ?? 'open'] ?? SHAPES.open)()}</g>;
}

/// Grid of round buttons to pick a gobo or the open beam
export function GoboButtons({
    gobos,
    selected,
    onSelect,
}: {
    gobos: { name: string; shape: string }[];
    selected: number | null;
    onSelect: (index: number | null) => void;
}) {
    const options = [{ name: 'Open', shape: null as string | null }, ...gobos];
    return (
        <div class="gobos">
            {options.map((gobo, index) => {
                const value = index === 0 ? null : index - 1;
                return (
                    <button
                        key={index}
                        class={`gobo ${value === selected ? 'is-selected' : ''}`}
                        title={gobo.name}
                        onClick={() => onSelect(value)}
                    >
                        <svg viewBox="-12 -12 24 24">
                            <circle class="gobo-disc" r={11} />
                            <GoboPattern shape={gobo.shape} />
                        </svg>
                    </button>
                );
            })}
        </div>
    );
}
