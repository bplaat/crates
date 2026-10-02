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

type Callback = (data: any) => void;

export class Ipc {
    type: IpcType;
    ws?: WebSocket;
    /// Callbacks by message type, so each message is parsed once whatever the number of listeners
    callbacks = new Map<string, Set<Callback>>();

    constructor() {
        const dispatch = (event: MessageEvent) => {
            const { type, ...data } = JSON.parse(event.data);
            const callbacks = this.callbacks.get(type);
            if (!callbacks) return;
            // Fixture outputs arrive every DMX frame, logging them slows down the page
            if (import.meta.env.MODE !== 'release' && type !== 'fixtureOutputs') console.debug(`Recv ${event.data}`);
            for (const callback of [...callbacks]) callback(data);
        };
        if ('ipc' in window) {
            this.type = 'ipc';
            window.ipc.addEventListener('message', dispatch);
        } else {
            this.type = 'websocket';
            this.ws = new WebSocket('/ipc');
            this.ws.addEventListener('message', dispatch);
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

    on(type: string, callback: Callback) {
        let callbacks = this.callbacks.get(type);
        if (!callbacks) this.callbacks.set(type, (callbacks = new Set()));
        // Wrap so the same function can be registered more than once
        const listener: Callback = (data) => callback(data);
        callbacks.add(listener);
        return {
            remove: () => void callbacks.delete(listener),
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
