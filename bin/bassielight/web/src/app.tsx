/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Route, Router } from './router.tsx';
import { StagePage } from './pages/stage.tsx';
import { SettingsPage } from './pages/settings.tsx';
import { NotFoundPage } from './pages/notfound.tsx';
import { Menubar } from './components/menubar.tsx';
import { EditorPage } from './pages/editor.tsx';

export function App() {
    return (
        <>
            <Menubar />

            <Router>
                <Route path="/" component={StagePage} />
                <Route path="/editor" component={EditorPage} />
                <Route path="/settings" component={SettingsPage} />
                <Route fallback component={NotFoundPage} />
            </Router>
        </>
    );
}
