import assert from 'node:assert/strict';
import { test } from 'node:test';
import { pointBox, snapDrag } from '../src/snap.ts';

const room = { width: 800, height: 1000 };
const button = { minX: 337.5, maxX: 462.5, minY: 475, maxY: 525 };

test('fixture snapping to an odd-width button keeps whole-centimeter positions', () => {
    const { delta } = snapDrag([pointBox(300, 500)], [button], room, { x: 38, y: 0 }, 2);
    assert.equal(delta.x, 38);
    assert.equal(delta.y, 0);
});

test('odd-width button stays inside the room at whole-centimeter positions', () => {
    for (const threshold of [null, 8]) {
        const { delta } = snapDrag([button], [], room, { x: -400, y: 0 }, threshold);
        assert.equal(delta.x, -337);
        assert.equal(button.minX + delta.x, 0.5);
        assert.ok(Number.isInteger(400 + delta.x));
    }
});

test('matching half-centimeter edges still align exactly', () => {
    const target = { ...button, minX: 537.5, maxX: 662.5 };
    const { delta, guides } = snapDrag([button], [target], room, { x: 74, y: 0 }, 2);
    assert.equal(delta.x, 75);
    assert.ok(guides.some((guide) => guide.axis === 'x' && guide.value === 537.5));
});
