/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import './index.css';
import { App } from './app.tsx';

render(<App />, document.getElementById('app')!);
