/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

const IpcType = {
    Ipc: 'ipc',
    WebSocket: 'websocket',
};
export { IpcType };

export default class Ipc {
    constructor() {
        if ('ipc' in window) {
            this.type = IpcType.Ipc;
        } else {
            this.type = IpcType.WebSocket;
            this.ws = new WebSocket('/ipc');
        }
    }

    send(type, data = {}) {
        const message = JSON.stringify({ type, ...data });
        return new Promise((resolve) => {
            if (this.type === IpcType.Ipc) {
                window.ipc.postMessage(message);
                resolve(undefined);
            }
            if (this.type === IpcType.WebSocket) {
                if (this.ws.readyState === WebSocket.OPEN) {
                    this.ws.send(message);
                    resolve(undefined);
                } else {
                    this.ws.addEventListener(
                        'open',
                        () => {
                            this.ws.send(message);
                            resolve(undefined);
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
            if (receivedType === type) callback(data);
        };
        if (this.type === IpcType.Ipc) window.ipc.addEventListener('message', listener);
        if (this.type === IpcType.WebSocket) this.ws.addEventListener('message', listener);
        return {
            remove: () => {
                if (this.type === IpcType.Ipc) window.ipc.removeEventListener('message', listener);
                if (this.type === IpcType.WebSocket) this.ws.removeEventListener('message', listener);
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
