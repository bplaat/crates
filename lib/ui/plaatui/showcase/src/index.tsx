/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import 'plaatui/base.css';
import './index.css';
import { render } from 'preact';
import { App } from './app.tsx';

render(<App />, document.getElementById('app')!);
