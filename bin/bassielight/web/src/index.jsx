/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import './index.css';
import { App } from './app.jsx';

window.addEventListener('contextmenu', (event) => event.preventDefault());

render(<App />, document.getElementById('app'));
