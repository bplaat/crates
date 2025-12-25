/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export class Ipc {
    send(type, data = {}) {
        const message = JSON.stringify({ type, ...data });
        return new Promise((resolve) => {
            window.ipc.postMessage(message);
            resolve(undefined);
        });
    }

    on(type, callback) {
        const listener = (event) => {
            const { type: receivedType, ...data } = JSON.parse(event.data);
            if (receivedType === type) callback(data);
        };
        window.ipc.addEventListener('message', listener);
        return {
            remove: () => window.ipc.removeEventListener('message', listener),
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
