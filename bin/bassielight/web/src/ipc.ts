/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

declare global {
    interface Window {
        ipc: EventTarget & {
            postMessage: (message: string) => void;
            addEventListener: (
                type: 'message',
                listener: (event: MessageEvent) => void,
                options?: boolean | AddEventListenerOptions,
            ) => void;
            removeEventListener: (
                type: 'message',
                listener: (event: MessageEvent) => void,
                options?: boolean | EventListenerOptions,
            ) => void;
        };
    }
}

export type IpcType = 'ipc' | 'websocket';

export class Ipc {
    type: IpcType;
    ws?: WebSocket;

    constructor() {
        if ('ipc' in window) {
            this.type = 'ipc';
        } else {
            this.type = 'websocket';
            this.ws = new WebSocket('/ipc');
        }
    }

    send(type: string, data: { [key: string]: any } = {}) {
        const message = JSON.stringify({ type, ...data });
        return new Promise((resolve) => {
            if (this.type === 'ipc') {
                window.ipc.postMessage(message);
                resolve(undefined);
            }
            if (this.type === 'websocket') {
                if (this.ws!.readyState === WebSocket.OPEN) {
                    this.ws!.send(message);
                    resolve(undefined);
                } else {
                    this.ws!.addEventListener(
                        'open',
                        () => {
                            this.ws!.send(message);
                            resolve(undefined);
                        },
                        { once: true },
                    );
                }
            }
        });
    }

    on(type: string, callback: (data: object) => void) {
        const listener = (event: MessageEvent) => {
            const { type: receivedType, ...data } = JSON.parse(event.data);
            if (receivedType === type) {
                console.debug(`Recv ${event.data}`);
                callback(data);
            }
        };
        if (this.type === 'ipc') window.ipc.addEventListener('message', listener);
        if (this.type === 'websocket') this.ws!.addEventListener('message', listener);
        return {
            remove: () => {
                if (this.type === 'ipc') window.ipc.removeEventListener('message', listener);
                if (this.type === 'websocket') this.ws!.removeEventListener('message', listener);
            },
        };
    }

    request(type: string, data: { [key: string]: any } = {}) {
        return new Promise((resolve) => {
            const listener = this.on(`${type}Response`, (data) => {
                listener.remove();
                resolve(data);
            });
            this.send(type, data);
        });
    }
}
