/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import 'plaatui/base.css';
import { render } from 'preact';
import './app.css';
import { App } from './app.tsx';

render(<App />, document.getElementById('app')!);
