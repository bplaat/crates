/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { render } from 'preact';
import 'plaatui/base.css';
import './index.css';
import { App } from './app.tsx';

render(<App />, document.getElementById('app')!);
