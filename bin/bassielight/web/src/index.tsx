/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import './index.css';
import { App } from './app.tsx';

if (import.meta.env.MODE === 'release') {
    window.addEventListener('contextmenu', (event) => event.preventDefault());
}

render(<App />, document.getElementById('app')!);
