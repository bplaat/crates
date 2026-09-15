/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import 'plaatui/base.css';
import { render } from 'preact';
import { App } from './app.tsx';
import './styles.css';

render(<App />, document.getElementById('app')!);

