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
        const message = JSON.stringify({ type, ...data });
        console.log(`[WEBV] Send ${message}`);
        return new Promise((resolve) => {
            if (this.type === 'ipc') {
                window.ipc.postMessage(message);
                resolve();
            }
            if (this.type === 'ws') {
                if (this.ws.readyState === WebSocket.OPEN) {
                    this.ws.send(message);
                    resolve();
                } else {
                    this.ws.addEventListener(
                        'open',
                        () => {
                            this.ws.send(message);
                            resolve();
                        },
                        { once: true }
                    );
                }
            }
        });
    }

    on(type, callback) {
        const listener = (event) => {
            const { type: receivedType, ...data } = JSON.parse(event.data);
            if (receivedType === type) {
                console.log(`[WEBV] Recv ${event.data}`);
                callback(data);
            }
        };
        if (this.type === 'ipc') window.ipc.addEventListener('message', listener);
        if (this.type === 'ws') this.ws.addEventListener('message', listener);
        return {
            remove: () => {
                if (this.type === 'ipc') window.ipc.removeEventListener('message', listener);
                if (this.type === 'ws') this.ws.removeEventListener('message', listener);
            },
        };
    }

    request(type, data = {}) {
        return new Promise((resolve) => {
            const listener = this.on(`${type}Response`, (data) => {
                listener.remove();
                resolve(data);
            });
            this.send(type, data);
        });
    }
}
