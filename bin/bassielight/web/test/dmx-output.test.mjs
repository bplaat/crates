import assert from 'node:assert/strict';
import { test } from 'node:test';
import { $dmxLive, syncDmxOutput } from '../src/stage.ts';

test('switching between Stage and Scripts keeps output running and updates setup freezing', () => {
    const messages = [];
    const ipc = { send: (type, data) => messages.push({ type, ...data }) };
    for (const location of ['/', '/scripts', '/']) {
        syncDmxOutput(ipc, location);
        assert.equal($dmxLive.value, true);
    }
    assert.deepEqual(messages, [
        { type: 'start', freeze: true },
        { type: 'start', freeze: false },
        { type: 'start', freeze: true },
    ]);
});

test('leaving a live page stops output and returning resumes it', () => {
    for (const destination of ['/editor', '/unknown']) {
        const messages = [];
        const ipc = { send: (type, data) => messages.push({ type, ...data }) };
        syncDmxOutput(ipc, '/scripts');
        syncDmxOutput(ipc, destination);
        assert.equal($dmxLive.value, false);
        syncDmxOutput(ipc, '/scripts');
        assert.equal($dmxLive.value, true);
        assert.deepEqual(messages, [
            { type: 'start', freeze: false },
            { type: 'stop' },
            { type: 'start', freeze: false },
        ]);
    }
});
