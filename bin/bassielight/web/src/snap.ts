/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// Axis aligned box, a fixture is a box without size
export interface Box {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
}

/// Alignment line shown while dragging, from and to are along the other axis
export interface Guide {
    axis: 'x' | 'y';
    value: number;
    from: number;
    to: number;
}

type Axis = 'x' | 'y';

export function pointBox(x: number, y: number): Box {
    return { minX: x, minY: y, maxX: x, maxY: y };
}

export function boundsBox(points: { x: number; y: number }[]): Box {
    const xs = points.map((p) => p.x);
    const ys = points.map((p) => p.y);
    return { minX: Math.min(...xs), minY: Math.min(...ys), maxX: Math.max(...xs), maxY: Math.max(...ys) };
}

const min = (box: Box, axis: Axis) => (axis === 'x' ? box.minX : box.minY);
const max = (box: Box, axis: Axis) => (axis === 'x' ? box.maxX : box.maxY);

/// Edges and center of a box along an axis
function anchors(box: Box, axis: Axis): number[] {
    return [...new Set([min(box, axis), Math.round((min(box, axis) + max(box, axis)) / 2), max(box, axis)])];
}

function moveBox(box: Box, delta: { x: number; y: number }): Box {
    return { minX: box.minX + delta.x, minY: box.minY + delta.y, maxX: box.maxX + delta.x, maxY: box.maxY + delta.y };
}

/// Snap a drag of the moving boxes to the edges and centers of the target boxes and the room, like
/// drawing apps do, keeping the moving boxes inside the room
export function snapDrag(
    moving: Box[],
    targets: Box[],
    room: { width: number; height: number },
    delta: { x: number; y: number },
    threshold: number | null,
): { delta: { x: number; y: number }; guides: Guide[] } {
    const roomBox: Box = { minX: 0, minY: 0, maxX: room.width, maxY: room.height };
    const allTargets = [...targets, roomBox];

    const snapAxis = (axis: Axis) => {
        // Range of offsets that keeps the moving boxes inside the room
        const lowest = -Math.min(...moving.map((box) => min(box, axis)));
        const highest = max(roomBox, axis) - Math.max(...moving.map((box) => max(box, axis)));
        const offset = Math.min(Math.max(delta[axis], lowest), highest);
        if (threshold === null) return offset;

        let best: number | null = null;
        for (const line of allTargets.flatMap((box) => anchors(box, axis))) {
            for (const value of moving.flatMap((box) => anchors(box, axis))) {
                const diff = line - (value + offset);
                if (Math.abs(diff) <= threshold && (best === null || Math.abs(diff) < Math.abs(best))) best = diff;
            }
        }
        const snapped = offset + (best ?? 0);
        return snapped >= lowest && snapped <= highest ? snapped : offset;
    };

    const snappedDelta = { x: snapAxis('x'), y: snapAxis('y') };
    if (threshold === null) return { delta: snappedDelta, guides: [] };

    // Show a guide for every aligned anchor, spanning all boxes on it
    const moved = moving.map((box) => moveBox(box, snappedDelta));
    const guides: Guide[] = [];
    for (const axis of ['x', 'y'] as Axis[]) {
        const other: Axis = axis === 'x' ? 'y' : 'x';
        for (const value of new Set(moved.flatMap((box) => anchors(box, axis)))) {
            const alignedTargets = allTargets.filter((box) => anchors(box, axis).includes(value));
            if (alignedTargets.length === 0) continue;
            const aligned = [...alignedTargets, ...moved.filter((box) => anchors(box, axis).includes(value))];
            guides.push({
                axis,
                value,
                from: Math.min(...aligned.map((box) => min(box, other))),
                to: Math.max(...aligned.map((box) => max(box, other))),
            });
        }
    }
    return { delta: snappedDelta, guides };
}
