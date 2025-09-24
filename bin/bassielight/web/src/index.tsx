/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import './index.css';
import { Ipc } from './ipc.ts';
import { App } from './app.tsx';

const ipc = new Ipc();
export { ipc };

if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
    document.body.classList.add('is-bwebview-macos');
}
if (import.meta.env.MODE === 'release') {
    window.addEventListener('contextmenu', (event) => event.preventDefault());
}

render(<App />, document.getElementById('app')!);
