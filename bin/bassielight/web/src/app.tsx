/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Route, Router } from './router.tsx';
import { Stage } from './pages/stage.tsx';
import { Settings } from './pages/settings.tsx';
import { NotFound } from './pages/notfound.tsx';
import { Menubar } from './components/menubar.tsx';

export function App() {
    return (
        <>
            <Menubar />

            <Router>
                <Route path="/" component={Stage} />
                <Route path="/settings" component={Settings} />
                <Route fallback component={NotFound} />
            </Router>
        </>
    );
}
