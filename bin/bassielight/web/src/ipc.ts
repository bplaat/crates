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
                options?: boolean | AddEventListenerOptions
            ) => void;
            removeEventListener: (
                type: 'message',
                listener: (event: MessageEvent) => void,
                options?: boolean | EventListenerOptions
            ) => void;
        };
    }
}

export enum IpcType {
    Ipc = 'ipc',
    WebSocket = 'websocket',
}

export class Ipc {
    type: IpcType;
    ws?: WebSocket;

    constructor() {
        if ('ipc' in window) {
            this.type = IpcType.Ipc;
        } else {
            this.type = IpcType.WebSocket;
            this.ws = new WebSocket('/ipc');
        }
    }

    send(type: string, data: { [key: string]: any } = {}) {
        const message = JSON.stringify({ type, ...data });
        console.debug(`Send ${message}`);
        return new Promise((resolve) => {
            if (this.type === IpcType.Ipc) {
                window.ipc.postMessage(message);
                resolve(undefined);
            }
            if (this.type === IpcType.WebSocket) {
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
                        { once: true }
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
        if (this.type === IpcType.Ipc) window.ipc.addEventListener('message', listener);
        if (this.type === IpcType.WebSocket) this.ws!.addEventListener('message', listener);
        return {
            remove: () => {
                if (this.type === IpcType.Ipc) window.ipc.removeEventListener('message', listener);
                if (this.type === IpcType.WebSocket) this.ws!.removeEventListener('message', listener);
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
