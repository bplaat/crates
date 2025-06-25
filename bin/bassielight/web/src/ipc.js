/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export default class Ipc {
    constructor() {
        if ('ipc' in window) {
            this.type = 'ipc';
        } else {
            this.type = 'ws';
            this.ws = new WebSocket('/ipc');
        }
    }

    send(type, data = {}) {
        console.log(`[WEBV] Send ${type}`);
        const message = JSON.stringify({ type, ...data });
        if (this.type === 'ipc') window.ipc.postMessage(message);
        if (this.type === 'ws') this.ws.send(message);
    }
}
