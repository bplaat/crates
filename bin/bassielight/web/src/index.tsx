/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import './index.css';
import { render } from 'preact';
import { App } from './app.tsx';

if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
    document.body.classList.add('is-bwebview-macos');
}
if (import.meta.env.MODE === 'release') {
    window.addEventListener('contextmenu', (event) => event.preventDefault());
}

render(<App />, document.getElementById('app')!);
