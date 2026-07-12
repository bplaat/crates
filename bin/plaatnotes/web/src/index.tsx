/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import 'plaatui/base.css';
import './app.css';
import { App } from './app.tsx';

render(<App />, document.getElementById('app')!);
